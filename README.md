# Space Travel — fase preliminar

Sistema solar explorable renderizado con un pipeline de software propio (Rust + raylib solo como
ventana/input/presentador del framebuffer). Esta es la fase base: sol, nave y 5 planetas
placeholder (esferas de color) con todo el esqueleto técnico funcionando (cámara, navegación,
gravedad, colisión, frustum culling, LOD). Los temas visuales por planeta (superficie, shaders
propios por planeta) se agregan en una fase posterior.

## Requisitos

- Rust (toolchain `stable-x86_64-pc-windows-gnu`) — instalado vía `rustup`.
- Un compilador C + CMake en el PATH (se usó MinGW-w64/WinLibs) y LLVM/libclang (para `bindgen`,
  que genera los bindings de raylib).
- Variables de entorno necesarias para compilar (ya configuradas como variables de usuario en esta
  máquina): `LIBCLANG_PATH` apuntando a `LLVM\bin`, `BINDGEN_EXTRA_CLANG_ARGS` con los `-I` de los
  headers de MinGW.

## Ejecutar

```
cargo run
```

## Controles

- Mouse: orientar la nave (pitch/yaw).
- W/S: empuje adelante/atrás (aceleración progresiva, no velocidad instantánea).
- A/D: roll.
- Shift (o clic derecho): boost.
- Space: frenar (amortiguación progresiva de la velocidad).
- Acercarse a un planeta activa automáticamente el "modo local" (velocidad máxima reducida para
  facilitar rodearlo de cerca).

## Nave

El modelo de la nave (`assets/models/ship.obj`, hecho en Blender) se carga con un parser de `.obj`
propio (`src/render/obj_loader.rs`) — no se usa el cargador de modelos de raylib. Si el archivo no
está presente, se usa una cuña de respaldo generada en código para que el proyecto siga siendo
ejecutable.

## Qué sigue

Tematizar cada uno de los 5 planetas (tipos de modelo de superficie, distribución procedural,
shaders propios por planeta) sin tocar la arquitectura central — el punto de extensión es
`render::shader::FragmentShader` por cuerpo.
