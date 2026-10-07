/* El puente entre Apolo y los main de avifenc, avifdec, cjxl y djxl, que se
 * compilan dentro con otro nombre (ADR 0021).
 *
 * Las herramientas de libjxl terminan con exit() ante una opción mala; dentro
 * de Apolo eso cerraría la aplicación. build.rs cambia esos exit() por
 * apolo_salir(), que vuelve aquí con longjmp. Lo que la herramienta tuviera
 * reservado se pierde: pasa solo con órdenes que la herramienta rechaza. */

#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>

#if defined(_WIN32)
#include <fcntl.h>
#include <io.h>
#define dup _dup
#define dup2 _dup2
#define close _close
#define NULO "NUL"
static int abrir_nulo(void) { return _open(NULO, _O_WRONLY); }
#else
#include <fcntl.h>
#include <unistd.h>
static int abrir_nulo(void) { return open("/dev/null", O_WRONLY); }
#endif

static _Thread_local jmp_buf * salto = NULL;

void apolo_salir(int codigo)
{
    if (salto) {
        longjmp(*salto, codigo ? codigo : 1);
    }
    exit(codigo);
}

typedef int (*Principal)(int, char **);

/* Con `callado`, lo que la herramienta escribe por la salida estándar se
 * tira: avifenc y avifdec no tienen --quiet, y dentro de Apolo no lo lee
 * nadie. Afecta a todo el proceso mientras dura, así que no pueden callarse
 * dos a la vez (Rust lo impide: avifenc y avifdec van con el mismo candado). */
static int ejecutar(Principal principal, int argc, char ** argv, int callado)
{
    jmp_buf aqui;
    jmp_buf * antes = salto;
    volatile int resultado;
    int copia = -1;
    if (callado) {
        fflush(stdout);
        int nulo = abrir_nulo();
        if (nulo >= 0) {
            copia = dup(1);
            if (copia >= 0) {
                dup2(nulo, 1);
            }
            close(nulo);
        }
    }
    salto = &aqui;
    int codigo = setjmp(aqui);
    if (codigo == 0) {
        resultado = principal(argc, argv);
    } else {
        resultado = codigo;
    }
    salto = antes;
    if (copia >= 0) {
        fflush(stdout);
        dup2(copia, 1);
        close(copia);
    }
    return resultado;
}

int apolo_avifenc_main(int argc, char ** argv);
int apolo_avifdec_main(int argc, char ** argv);
int apolo_cjxl_main(int argc, char ** argv);
int apolo_djxl_main(int argc, char ** argv);

int apolo_avifenc(int argc, char ** argv, int callado) { return ejecutar(apolo_avifenc_main, argc, argv, callado); }
int apolo_avifdec(int argc, char ** argv, int callado) { return ejecutar(apolo_avifdec_main, argc, argv, callado); }
int apolo_cjxl(int argc, char ** argv, int callado) { return ejecutar(apolo_cjxl_main, argc, argv, callado); }
int apolo_djxl(int argc, char ** argv, int callado) { return ejecutar(apolo_djxl_main, argc, argv, callado); }
