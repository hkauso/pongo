test:
    just style
    cargo run --example basic

style:
    bunx tailwindcss -i ./static/input.css -o ./static/style.css
