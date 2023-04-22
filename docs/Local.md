# Local

ccache defaults to using local disk storage. You can set the `CCACHE_DIR` environment variable to change the disk cache location. By default it will use a sensible location for the current platform: `~/.cache/ccache` on Linux, `%LOCALAPPDATA%\Shediao\ccache` on Windows, and `~/Library/Caches/Shediao.ccache` on MacOS.

The default cache size is 10 gigabytes. To change this, set `CCACHE_CACHE_SIZE`, for example `CCACHE_CACHE_SIZE="1G"`.

The local storage only supports a single ccache server at a time. Multiple concurrent servers will race and cause spurious build failures.
