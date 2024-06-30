const table_view = reg_ns("table_view");

table_view.define("render", async function ({ $ }, bind_to, table_name) {
    // get params
    const search = new URLSearchParams(window.location.search);

    let query = search.get("query");
    let mode = search.get("mode");

    if (!query || !mode) {
        query = "SELECT * FROM $table LIMIT 100";
        mode = "fetch";
    }

    // create context menu
    $.ctx = document.createElement("div");
    $.ctx.setAttribute("class", "flex flex-col link-list elevated round");
    $.ctx.style.animation = "fadein 0.05s ease-in-out 1 running";
    $.ctx.style.width = "15rem";

    window.addEventListener("contextmenu", (e) => {
        if (e.target && e.target.nodeName === "INPUT") {
            return;
        }

        e.preventDefault();
        $.context_menu(e);
    });

    window.addEventListener("click", () => {
        $.ctx.remove();
    });

    // query builder
    globalThis.set_selector = (selector) => {
        for (const field of Array.from(
            document.querySelectorAll('input[name="selector"]'),
        )) {
            field.value = selector;
        }

        document.getElementById("add_to_query_dialog").showModal();
    };

    globalThis.set_match = (idx, col) => {
        const value = $.payload[idx][col]; // get value from [row][col]

        for (const field of Array.from(
            document.querySelectorAll('input[name="match"]'),
        )) {
            field.value = value;
        }

        document.getElementById("add_to_query_dialog").showModal();
    };

    globalThis.add_to_query = (keyword, e) => {
        e.preventDefault();
        document.getElementById("query").value +=
            ` ${keyword.toUpperCase()} ${e.target.value.value}`;
        document.getElementById("add_to_query_dialog").close();
        e.target.reset();
    };

    globalThis.add_to_query_eq = (keyword, e) => {
        e.preventDefault();
        document.getElementById("query").value +=
            ` ${keyword.toUpperCase()} \"${e.target.selector.value.replaceAll('"', "'")}\" = '${e.target.match.value.replaceAll("'", '"')}'`;
        document.getElementById("add_to_query_dialog").close();
        e.target.reset();
    };

    globalThis.add_to_query_neq = (keyword, e) => {
        e.preventDefault();
        document.getElementById("query").value +=
            ` ${keyword.toUpperCase()} \"${e.target.selector.value.replaceAll('"', "'")}\" != '${e.target.match.value.replaceAll("'", '"')}'`;
        document.getElementById("add_to_query_dialog").close();
        e.target.reset();
    };

    globalThis.update_query = (e) => {
        e.preventDefault();

        // get selected columns
        let selected_columns = [];

        for (const col of columns) {
            if (document.getElementById(`col:${col}`).checked === false) {
                continue;
            }

            selected_columns.push(col);
        }

        // build output
        let output = "UPDATE $table SET ";

        for (const [i, col] of Object.entries(selected_columns)) {
            let ending = ",";

            if (parseInt(i) === selected_columns.length - 1) {
                ending = "";
            }

            // get value
            const value = document.getElementById(`colv:${col}`).value;

            if (!value) {
                continue;
            }

            // ...
            output += `"${col}" = '${value}'${ending}`;
        }

        // set query
        document.getElementById("query").value = output;

        // set mode to execute
        document
            .querySelector('option[value="execute"]')
            .setAttribute("selected", "true");

        // hide dialog
        document.getElementById("update_builder_dialog").close();

        // show specifier dialog
        document.getElementById("add_to_query_dialog").showModal();
    };

    // fill input fields
    document.getElementById("query").value = query;

    if (mode === "execute") {
        document
            .querySelector('option[value="execute"]')
            .setAttribute("selected", "true");
    }

    // warnings
    if (query.includes("DELETE") && !query.includes("LIMIT")) {
        if (
            !confirm(
                "You're attempting to execute a DELETE query with no limit. Are you sure you want to do this?",
            )
        ) {
            trigger("app:gen_secret", [
                "note-info",
                "Request Canceled (DELETE)",
                "Make sure all your requests are safe before attempting to send them.",
            ]);

            return;
        }
    } else if (query.includes("UPDATE") && !query.includes("LIMIT")) {
        if (
            !confirm(
                "You're attempting to execute an UPDATE query with no limit. Are you sure you want to do this?",
            )
        ) {
            trigger("app:gen_secret", [
                "note-info",
                "Request Canceled (UPDATE)",
                "Make sure all your requests are safe before attempting to send them.",
            ]);

            return;
        }
    }

    // send request
    const res = await (
        await fetch(
            `/${globalThis._app_base.nested}/api/sql/${table_name}/${mode}`,
            {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    query,
                }),
            },
        )
    ).json();

    if (res.success === false) {
        trigger("app:gen_secret", [
            "note-error",
            "Invalid Request",
            res.message,
        ]);

        return;
    }

    if (!res.payload || res.payload.length === 0) {
        trigger("app:gen_secret", [
            "note-note",
            "Request Finished",
            "Query completed with no results.",
        ]);

        return;
    } else {
        trigger("app:gen_secret", [
            "note-note",
            "Request Finished",
            `Query completed with <b>${res.payload.length}</b> results.`,
        ]);
    }

    // build head
    bind_to.innerHTML += '<thead><tr id="table_head"></tr></thead>';

    const head = document.getElementById("table_head");
    const columns = Object.keys(res.payload[0]).sort((a, b) =>
        a.localeCompare(b),
    );

    for (const column of columns) {
        // update table
        head.innerHTML += `<th data-col="${column}">${column}</th>`;

        // update #cols
        document.getElementById("cols").innerHTML +=
            `<div style="display: flex; justify-content: space-between; align-items: center;">
<label for="col:${column}"><code>${column}</code></label>
<input 
    type="checkbox" 
    name="col:${column}" 
    id="col:${column}"
    class="w-max"
    onchange="document.getElementById('colv:${column}').toggleAttribute('disabled');"
/></div>`;

        document.getElementById("col_vals").innerHTML +=
            `<input type="text" name="colv:${column}" id="colv:${column}" placeholder="${column}" disabled />`;
    }

    // build body
    bind_to.innerHTML += '<tbody id="table_body"></tbody>';

    const body = document.getElementById("table_body");
    for (const [i, row] of Object.entries(res.payload)) {
        let output = "";

        for (const column of columns) {
            const value = row[column];

            if (value.length > 100) {
                // show value in dropdown
                const clean_value = value
                    .replaceAll("<", "&lt;")
                    .replaceAll(">", "&gt;")
                    .replaceAll('"', "&quot;");

                output += `<td data-col="${column}" data-idx="${i}" style="white-space: wrap;">
                    <details class="small secondary w-max" style="white-space: wrap;">
                        <summary>
                            <span>Long Value (${value.length})</span>
                        </summary>
                        
                        <span style="white-space: pre-wrap;">${clean_value}</span>
                    </details>`;
            } else {
                // just show value
                output += `<td data-col="${column}" data-idx="${i}">${value
                    .replaceAll("<", "&lt;")
                    .replaceAll(">", "&gt;")
                    .replaceAll('"', "&quot;")}</td>`;
            }
        }

        body.innerHTML += `<tr>${output}</tr>`;
    }

    $.payload = res.payload;
});

function read_selected() {
    let text = "";

    if (window.getSelection) {
        text = window.getSelection().toString();
    } else if (document.selection && document.selection.type != "Control") {
        text = document.selection.createRange().text;
    }

    return text;
}

table_view.define("context_menu", function ({ $ }, event) {
    $.selected = event.target;

    // move context menu
    $.ctx.style.position = "absolute";
    $.ctx.style.top = `${event.pageY}px`;
    $.ctx.style.left = `${event.pageX}px`;
    $.ctx.innerHTML = "";
    document.body.appendChild($.ctx);

    // populate options

    // text options
    const selection = read_selected();

    if (selection !== "") {
        $.ctx.innerHTML += `<button 
            class="w-full round green option small"
            onclick="trigger('table_view:copy_selection')"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="lucide lucide-copy"
                aria-label="Copy symbol"
            >
                <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
                <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
            </svg>
            
            Copy Selection
        </button>`;
    }

    // ...
    if (event.target) {
        // button
        if (
            event.target.nodeName === "BUTTON" ||
            event.target.nodeName === "A"
        ) {
            $.ctx.innerHTML += `<button 
                class="w-full round option small"
                onclick="trigger('table_view:click_target')"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    class="lucide lucide-mouse-pointer-click"
                    aria-label="Pointer click symbol"
                >
                    <path d="m9 9 5 12 1.8-5.2L21 14Z" />
                    <path d="M7.2 2.2 8 5.1" />
                    <path d="m5.1 8-2.9-.8" />
                    <path d="M14 4.1 12 6" />
                    <path d="m6 12-1.9 2" />
                </svg>
                                
                Activate
            </button>`;
        }

        // column
        if (event.target.getAttribute("data-col")) {
            if (event.target.getAttribute("data-idx")) {
                // fill WITH value
                $.ctx.innerHTML += `<button 
                    class="w-full round blue option small"
                    onclick="set_selector('${event.target.getAttribute("data-col")}');set_match(${event.target.getAttribute("data-idx")},'${event.target.getAttribute("data-col")}')"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="18"
                        height="18"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="lucide lucide-plus"
                        aria-label="Plus symbol"
                    >
                        <path d="M5 12h14" />
                        <path d="M12 5v14" />
                    </svg>
                        
                    Add Specifier
                </button>`;
            } else {
                // fill WITHOUT value
                $.ctx.innerHTML += `<button 
                    class="w-full round blue option small"
                    onclick="set_selector('${event.target.getAttribute("data-col")}')"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="18"
                        height="18"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="lucide lucide-plus"
                        aria-label="Plus symbol"
                    >
                        <path d="M5 12h14" />
                        <path d="M12 5v14" />
                    </svg>
                        
                    Add Specifier
                </button>`;
            }
        }

        // default options
        $.ctx.innerHTML += `<button
            class="w-full round red option small"
            onclick="window.location.reload()"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="lucide lucide-refresh-ccw"
                aria-label="Refresh symbol (ccw)"
            >
                <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
                <path d="M3 3v5h5" />
                <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
                <path d="M16 16h5v5" />
            </svg>
                                                    
            Repeat Query
        </button>`;

        $.ctx.innerHTML += `<button
            class="w-full round option small"
            onclick="window.location.href = '/${globalThis._app_base.nested}'"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="lucide lucide-table-2"
                aria-label="Table symbol"
            >
                <path
                    d="M9 3H5a2 2 0 0 0-2 2v4m6-6h10a2 2 0 0 1 2 2v4M9 3v18m0 0h10a2 2 0 0 0 2-2V9M9 21H5a2 2 0 0 1-2-2V9m0 0h18"
                />
            </svg>
                                        
            Change Table
        </button>`;
    }
});

table_view.define("copy_selection", function (_) {
    window.navigator.clipboard.writeText(read_selected());
});

table_view.define("click_target", function ({ $ }) {
    $.selected.click();
});
