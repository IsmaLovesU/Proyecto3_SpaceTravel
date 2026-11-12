# Space Travel

Sistema solar explorable renderizado con un pipeline de software propio (Rust + raylib solo como
ventana/input/presentador del framebuffer). Sol, nave y 5 planetas temáticos, cada uno con 4 tipos
de modelo de superficie exclusivos distribuidos proceduralmente (semilla reproducible), orientados
correctamente sobre la curvatura mientras el planeta rota:

1. **Reino Fungi Orbital** (Mario Galaxy) — hongo gigante, bloque "?", tubería, estrella.
2. **Tikal Prime** (Guatemala futurista) — pirámide escalonada con proyector, estela de jade,
   quetzal mecánico, panel solar con patrón textil. Shader propio `JadeCircuitShader` (rejilla
   tipo circuito/glifo brillante).
3. **Ruta Orbital** (Pokémon) — Poké Ball, árbol de bayas, piedra evolutiva, Centro Pokémon mini.
   Shader propio `BumpMappedShader` con **mapeo de normales** procedural (sin archivos de imagen).
4. **Laberinto Neón** (Pac-Man) — muro de laberinto, Pac-Dot, power pellet, fantasma. Shader propio
   `NeonGridShader` (rejilla neón pulsante con el tiempo).
5. **Mictlán Celestial** (Día de Muertos) — calavera de azúcar, alebrije, ofrenda con velas, arco
   de cempasúchil, sobre una superficie **deformada por ruido** (ya no es una esfera perfecta; los
   props se asientan sobre ese mismo terreno, no sobre la esfera original).

Más el sol (`EmissiveCoronaShader`, animado) y Mario reusando `LambertShader` — en total **4
shaders propios distintos**, cámara, navegación con gravedad y colisión, frustum culling y LOD por
distancia, todo con pipeline, matemática y rasterización propios (sin las funciones
3D/modelos/cámara/shaders de raylib). La nave se suaviza con una **etapa propia de subdivisión
procedural de geometría** (punto medio + suavizado Laplaciano) al cargarla. Con esto el menú de
puntos del curso está cubierto completo (200/200 brutos; nota = min(100, puntos)).

## Requisitos

- Rust (toolchain `stable-x86_64-pc-windows-gnu`) — instalado vía `rustup`.
- Un compilador C + CMake en el PATH (se usó MinGW-w64/WinLibs) y LLVM/libclang (para `bindgen`,
  que genera los bindings de raylib).
- Variables de entorno necesarias para que `raylib-sys` genere los bindings con `bindgen`/libclang:
  `LIBCLANG_PATH` (carpeta `bin` de LLVM) y `BINDGEN_EXTRA_CLANG_ARGS` (rutas `-I` a los headers de
  MinGW). Ya están fijadas en [`.cargo/config.toml`](.cargo/config.toml) con `force = true`, así que
  **no dependen de la terminal que uses** — si compilas en otra máquina, solo actualiza las rutas de
  ese archivo para que apunten a tu propio MinGW-w64/LLVM (usa `/` en vez de `\`: un `\` dentro del
  valor de una variable de entorno puede ser interpretado como carácter de escape y romper la ruta).

## Ejecutar

```
cargo run --release
```

El build de **debug** (`cargo run`) es notablemente más lento (rasterizador por software, mucho
cálculo por píxel) — usar siempre `--release` para la demo.

## Controles

- Mouse: orientar la nave (pitch/yaw).
- W/S: empuje adelante/atrás (aceleración progresiva, no velocidad instantánea).
- A/D: roll.
- Shift (o clic derecho): boost.
- Space: frenar (amortiguación progresiva de la velocidad).
- R: regenera la semilla de distribución procedural de superficie (reproducible: misma semilla =
  mismas posiciones).
- Acercarse a un planeta activa automáticamente el "modo local" (velocidad máxima reducida para
  facilitar rodearlo de cerca).

## Nave

El modelo de la nave (`assets/models/ship.obj`, hecho en Blender) se carga con un parser de `.obj`
propio (`src/render/obj_loader.rs`) — no se usa el cargador de modelos de raylib. Si el archivo no
está presente, se usa una cuña de respaldo generada en código para que el proyecto siga siendo
ejecutable.

## Props de superficie

Los 20 tipos de modelo (`src/scene/props.rs`) se construyen proceduralmente a partir de primitivas
propias (`src/render/primitives.rs`: caja/cilindro/cono) y la icosfera ya existente — nada de
Blender. Se distribuyen con una semilla reproducible (`src/scene/distribution.rs`, muestreo por
rechazo sobre la esfera) y rotan/orbitan automáticamente con el planeta porque su transformación se
compone sobre `body.model_matrix()`.

## Renderizado avanzado (dónde verlo)

- **Mapeo de normales**: planeta Ruta Orbital (Pokémon) — superficie con textura "moteada" en vez
  de lisa. `src/render/normal_map.rs` (mapa generado con ruido, sin archivos de imagen) +
  `BumpMappedShader` en `src/render/shader.rs`.
- **Deformación de geometría por ruido**: planeta Mictlán Celestial (Día de Muertos) — ya no es una
  esfera perfecta. `src/render/deform.rs` + `src/math/noise.rs` (ruido fractal propio). Los props
  de ese planeta usan la misma función de altura para no quedar flotando/enterrados.
- **Subdivisión procedural de geometría**: la nave, suavizada una vez al cargarla.
  `src/render/subdivision.rs` (subdivisión por punto medio + suavizado Laplaciano, genérica — sirve
  sobre cualquier malla, no solo esferas; distinta de la subdivisión de icosaedro usada para LOD).
