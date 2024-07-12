(() => {
    const self = reg_ns("starstraw");

    const error = document.getElementById("error");
    const success = document.getElementById("success");
    const forms = document.getElementById("forms");

    self.define("render_login", function (_, bind_to, callback) {
        bind_to.addEventListener("submit", async (e) => {
            e.preventDefault();
            const res = await fetch(
                `${globalThis._app_base.starstraw}/api/return`,
                {
                    method: "POST",
                    body: JSON.stringify({
                        id: e.target.uid.value,
                    }),
                    headers: {
                        "Content-Type": "application/json",
                    },
                },
            );

            const json = await res.json();

            if (json.success === false) {
                error.style.display = "block";
                error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
            } else {
                success.style.display = "flex";
                success.innerHTML = `<p>Successfully logged into account.</p>
                
                <hr />
                <a href="${callback}?uid=${json.message}" class="button round theme:primary">Continue</a>`;
                forms.style.display = "none";
            }
        });
    });

    self.define("render_register", function (_, bind_to, callback) {
        bind_to.addEventListener("submit", async (e) => {
            e.preventDefault();
            const res = await fetch(
                `${globalThis._app_base.starstraw}/api/start`,
                {
                    method: "POST",
                    body: JSON.stringify({
                        username: e.target.username.value,
                    }),
                    headers: {
                        "Content-Type": "application/json",
                    },
                },
            );

            const json = await res.json();

            if (json.success === false) {
                error.style.display = "block";
                error.innerHTML = `<div class="mdnote-title">${json.message}</div>`;
            } else {
                success.style.display = "flex";
                success.innerHTML = `<p>Account created! You can login using this code:</p>

                <p class="card secondary round flex justify-center align-center">${json.message}</p>

                <p><b>Do not lose it!</b> This code is required for you to sign into your account, <b>it cannot be reset!</b></p>
                
                <hr />
                <a href="${callback}?uid=${json.message}" class="button round theme:primary">Continue</a>`;
                forms.style.display = "none";
            }
        });
    });
})();
