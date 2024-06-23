publish:
    cargo publish --no-verify --allow-dirty

test:
    just style
    cargo run --example basic

style:
    bunx tailwindcss -i ./static/input.css -o ./static/style.css
