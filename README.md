# Space Travel

Sistema solar explorable renderizado con un pipeline de software propio (Rust + raylib solo como
ventana/input/presentador del framebuffer). Sol, nave y 5 planetas temáticos, cada uno con 4 tipos
de modelo de superficie exclusivos distribuidos proceduralmente (semilla reproducible), orientados
correctamente sobre la curvatura mientras el planeta rota:

1. **Reino Fungi Orbital** (Mario Galaxy) — hongo gigante con pecas y ojos, bloque "?" con
   signos en cada cara, tubería con planta piraña, estrella de poder de cinco puntas. Superficie
   de pasto caricaturesco con parches de tierra y lagunas.
2. **Tikal Prime** (Guatemala futurista) — pirámide escalonada con escalinata, luces de jade y
   antena holográfica; estela de jade con glifos encendidos; quetzal mecánico sobre un mástil;
   panel solar con patrón de huipil. Shader propio `JadeCircuitShader` (trazas de circuito/glifo
   brillantes sobre piedra de jade).
3. **Ruta Orbital** (Pokémon) — Poké Ball con botón, árbol de bayas, piedra evolutiva flotando
   sobre un pedestal, Centro Pokémon con letrero "P". Superficie con rutas de tierra, lagos y
   bosques. Shader propio `BumpMappedShader` con **mapeo de normales** procedural.
4. **Laberinto Neón** (Pac-Man) — muros de laberinto en L con borde neón, Pac-Dots, power
   pellets y fantasmas con ojos. Shader propio `NeonGridShader` (rejilla neón pulsante).
5. **Mictlán Celestial** (Día de Muertos) — calavera de azúcar con pétalos y dientes, alebrije
   con cuernos y alas, ofrenda de tres niveles con velas y foto, arco de cempasúchil con papel
   picado. Superficie **deformada por ruido** y coloreada por altura (valles violeta, laderas
   ocre, cumbres color cempasúchil).

Más el sol (`EmissiveCoronaShader`, animado, con halo en pantalla) — en total **4 shaders propios
distintos**, cámara, navegación con gravedad y colisión, frustum culling y LOD por distancia, todo
con pipeline, matemática y rasterización propios (sin las funciones 3D/modelos/cámara/shaders de
raylib). La nave se suaviza con una **etapa propia de subdivisión procedural de geometría** al
cargarla. Con esto el menú de puntos del curso está cubierto completo (200/200 brutos; nota =
min(100, puntos)).

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

> Si Windows muestra "Una directiva de Control de aplicaciones bloqueó este archivo" al lanzar el
> ejecutable, es **Control inteligente de aplicaciones** (Smart App Control) bloqueando un `.exe`
> sin firma recién compilado; no es un error del proyecto. Se resuelve desde Seguridad de Windows →
> Control de aplicaciones y navegador → Control inteligente de aplicaciones.

## Controles

- Mouse: orientar la nave (pitch/yaw). El giro pasa por una velocidad angular filtrada (no por el
  delta crudo del mouse), con tope de velocidad de giro — por eso se siente estable y no
  "tiembla". La nave y la cámara se inclinan (bank) al girar.
- W/S: empuje adelante/atrás (aceleración progresiva). La nave solo avanza mientras la tecla
  está presionada: al soltarla frena sola y queda quieta (la gravedad solo actúa mientras se
  mueve). La velocidad se va alineando con el morro, así vuela hacia donde apunta.
- A/D: roll.
- Shift (o clic derecho) con W: boost. Activa el efecto de "velocidad luz" (estelas radiales de
  las estrellas, viñeta azul y FOV más abierto) en src/render/sky.rs. Es solo visual: la velocidad
  máxima no cambia.
- Space: frenar (amortiguación progresiva).
- R: regenera la semilla de distribución procedural de superficie (reproducible: misma semilla =
  mismas posiciones).
- H: oculta/muestra el HUD y las etiquetas de los planetas (útil para el video). Cada cuerpo
  lleva una etiqueta 2D con su nombre y la distancia a su superficie, anclada sobre su borde
  superior proyectado (se oculta si otro cuerpo lo tapa).
- Acercarse a un planeta activa automáticamente el "modo local" (velocidad máxima reducida,
  con transición suave) para rodearlo de cerca. La gravedad curva la trayectoria cerca de los
  cuerpos; chocar con uno detiene la componente de velocidad hacia él.

## Escena

Órbitas amplias y lentas (periodos de ~2 a ~8 minutos) para que cada planeta se aprecie como
destino propio y no se "escape" a medio acercamiento. El fondo es un campo de estrellas
procedural proyectado por el mismo pipeline (puntos al infinito, sin paralaje) y el sol lleva un
halo aditivo en espacio de pantalla, ambos en `src/render/sky.rs`.

## Nave

El modelo de la nave (`assets/models/ship.obj`, hecho en Blender) se carga con un parser de `.obj`
propio (`src/render/obj_loader.rs`) — no se usa el cargador de modelos de raylib. El casco se
repinta por vértice (casco claro, panza oscura, banda y punta naranja, zona de cabina azul) y se
le añaden piezas procedurales con las mismas primitivas de los props: dos motores con núcleo
brillante, cúpula de cabina, aleta dorsal con mástil, winglets, luces de navegación (roja/verde),
faro de proa, tomas de aire y patines (`src/scene/ship_style.rs`). Antes de
subdividirla se sueldan los vértices duplicados del `.obj` (`weld_vertices` en
`src/render/subdivision.rs`) para que el suavizado no abra grietas. Si el archivo no está
presente, se usa una cuña de respaldo generada en código para que el proyecto siga siendo
ejecutable.

## Props de superficie

Los 20 tipos de modelo (`src/scene/props.rs`) se construyen proceduralmente a partir de primitivas
propias (`src/render/primitives.rs`: caja/cilindro/cono) y la icosfera ya existente — nada de
Blender. Se distribuyen con una semilla reproducible (`src/scene/distribution.rs`, muestreo por
rechazo sobre la esfera) y rotan/orbitan automáticamente con el planeta porque su transformación se
compone sobre `body.model_matrix()`. Cada planeta tiene además un `prop_shader` propio (Lambert
en general; rejilla neón gruesa en Pac-Man para que los muros tengan bordes brillantes), porque
los shaders de rejilla/bump del planeta están afinados para las UV de la esfera y se ven como
ruido sobre cajas pequeñas.

## Rendimiento

- Frustum culling por cuerpo (esfera vs. 6 planos) y por instancia de prop.
- LOD por distancia con 5 niveles de icosfera (20 → 5120 triángulos).
- Props: solo se dibujan a menos de 22 radios del planeta, se descartan los del hemisferio
  oculto (`normal · (cámara − pos) < 0`, elimina ~la mitad) y los que proyectarían menos de 3 px.
- Rasterizador: recorte real contra el plano cercano (sin triángulos gigantes al rozar un
  planeta), rechazo trivial por plano de clip, funciones de borde incrementales por fila y pesos
  de perspectiva normalizados una sola vez por píxel. El halo del sol usa una tabla de atenuación.
- El HUD muestra cuerpos/props visibles y triángulos enviados / descartados por back-face /
  rasterizados.

## Renderizado avanzado (dónde verlo)

- **Mapeo de normales**: planeta Ruta Orbital (Pokémon) — superficie con textura "moteada" en vez
  de lisa. `src/render/normal_map.rs` (mapa generado con ruido, sin archivos de imagen) +
  `BumpMappedShader` en `src/render/shader.rs`.
- **Deformación de geometría por ruido**: planeta Mictlán Celestial (Día de Muertos) — ya no es una
  esfera perfecta. `src/render/deform.rs` + `src/math/noise.rs` (ruido fractal propio). Los props
  de ese planeta usan la misma función de altura para no quedar flotando/enterrados, y la colisión
  usa el radio con relieve.
- **Subdivisión procedural de geometría**: la nave, suavizada una vez al cargarla.
  `src/render/subdivision.rs` (soldado de vértices + subdivisión por punto medio + suavizado
  Laplaciano, genérica — sirve sobre cualquier malla, no solo esferas; distinta de la subdivisión
  de icosaedro usada para LOD).
- **Coloreado procedural por vértice**: cada planeta tiene una paleta de biomas evaluada con ruido
  sobre la esfera (`*_surface_color` en `src/scene/solar_system.rs`), sin costo por frame.
