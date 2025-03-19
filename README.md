# 3D Spinning Cube with Rust and WebAssembly

This project demonstrates how to use Rust and WebAssembly to render a 3D spinning cube using WebGL. The cube is rendered with different colors for each face and rotates continuously.

![Screenshot of the 3D Spinning Cube](screenshot.png)

## Demo

A live demo of this project is available at: [https://yourusername.github.io/p003-3d-cube-github-pages/](https://yourusername.github.io/p003-3d-cube-github-pages/)

## Technologies Used

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [WebAssembly (Wasm)](https://webassembly.org/) - Binary instruction format for a stack-based virtual machine
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) - Facilitating high-level interactions between Wasm modules and JavaScript
- [web-sys](https://docs.rs/web-sys/) - Raw bindings to Web APIs
- [WebGL](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API) - JavaScript API for rendering interactive 3D graphics

## Features

- 3D cube rendering with WebGL
- Different colors for each face of the cube
- Continuous rotation animation
- Controls to start and stop the animation
- Responsive design

## Local Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) - Ensure you have Rust installed
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) - For building the WebAssembly package

### Building the Project

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/p003-3d-cube-github-pages.git
   cd p003-3d-cube-github-pages
   ```

2. Build the WebAssembly package:
   ```bash
   wasm-pack build --target web
   ```

3. Serve the project locally using a web server:
   ```bash
   # Using Python 3
   python -m http.server
   
   # OR using Node.js and npx
   npx serve
   ```

4. Open your browser and navigate to `http://localhost:8000` (or whatever port your server uses)

## Project Structure

- `src/lib.rs` - Rust code for rendering the 3D cube
- `index.html` - HTML page that loads the WebAssembly and displays the cube
- `style.css` - Styling for the webpage
- `.github/workflows/deploy.yml` - GitHub Actions workflow for deploying to GitHub Pages

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- This project was inspired by the [Rust and WebAssembly](https://rustwasm.github.io/docs/book/) book
- Thanks to the Rust and WebAssembly communities for their excellent documentation and resources 