# Lamina

Reusable immediate-mode wgpu UI toolkit and design system, shared by Terra (landscape) and Visage (face animation).

Lamina is domain-neutral: no terrain or character types, no winit, no product chrome. Apps own the event loop and compose a 3D (or other) pass first, then `GuiRenderer::render` with `LoadOp::Load`, then present once.

## Depend via path

From a sibling checkout (this repo at `D:\GameDevTooling\Lamina`):

```toml
lamina = { path = "../Lamina/crates/lamina" }
```

Pin **wgpu 24** to match this crate. Lamina does not depend on winit; if the app uses winit, keep **winit 0.30**.

## Workspace

| Crate | Role |
|-------|------|
| `lamina` | Immediate-mode wgpu UI toolkit and design system |
| `lamina-test-gpu` | Headless GPU harness for GUI tests (dev-only; not published) |

```bash
cargo test -p lamina
cargo check -p lamina
```

GPU tests skip when no adapter is available.

## License

MIT. See [LICENSE](LICENSE).
