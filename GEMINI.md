5. ESTÁNDARES DE ARQUITECTURA Y CLEAN CODE (RUST):
- Principio de Responsabilidad Única (SRP): Cada módulo o función debe tener un único propósito absoluto.
- Límites de Archivo: Ningún archivo (`.rs`) debe superar las 400 líneas de código.
- Límites de Función: Ninguna función debe superar las 30 líneas.
- Densidad: Máximo 10-15 funciones por archivo.
- Acción Obligatoria: Si la solución requiere superar estos límites, el agente DEBE proponer la creación de submódulos (usando `mod.rs`) y separar la lógica en lugar de crear archivos monolíticos.