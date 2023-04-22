#include <stdio.h>

#if !defined(CCACHE_TEST_DEFINE)
#error CCACHE_TEST_DEFINE is not defined
#endif

void foo() { printf("hello world\n"); }
