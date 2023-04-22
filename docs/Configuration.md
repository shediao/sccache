# Available Configuration Options

## file

```toml
[dist]
# where to find the scheduler
scheduler_url = "http://1.2.3.4:10600"
# a set of prepackaged toolchains
toolchains = []
# the maximum size of the toolchain cache in bytes
toolchain_cache_size = 5368709120
cache_dir = "/home/user/.cache/ccache-dist-client"

[dist.auth]
type = "token"
token = "secrettoken"


#[cache.azure]
# does not work as it appears

[cache.disk]
dir = "/tmp/.cache/ccache"
size = 7516192768 # 7 GiBytes

[cache.gcs]
# optional oauth url
oauth_url = "..."
# optional deprecated url
deprecated_url = "..."
rw_mode = "READ_ONLY"
# rw_mode = "READ_WRITE"
cred_path = "/psst/secret/cred"
bucket = "bucket"
key_prefix = "prefix"

[cache.gha]
url = "http://localhost"
token = "secret"
cache_to = "ccache-latest"
cache_from = "ccache-"

[cache.memcached]
url = "..."

[cache.redis]
url = "redis://user:passwd@1.2.3.4:6379/1"

[cache.s3]
bucket = "name"
endpoint = "s3-us-east-1.amazonaws.com"
use_ssl = true
key_prefix = "s3prefix"
```

## env

Whatever is set by a file based configuration, it is overruled by the env
configuration variables

### misc

* `CCACHE_ALLOW_CORE_DUMPS` to enable core dumps by the server
* `CCACHE_CONF` configuration file path
* `CCACHE_CACHED_CONF`
* `CCACHE_IDLE_TIMEOUT` how long the local daemon process waits for more client requests before exiting, in seconds. Set to `0` to run ccache permanently
* `CCACHE_STARTUP_NOTIFY` specify a path to a socket which will be used for server completion notification
* `CCACHE_MAX_FRAME_LENGTH` how much data can be transferred between client and server
* `CCACHE_NO_DAEMON` set to `1` to disable putting the server to the background

### cache configs

#### disk

* `CCACHE_DIR` local on disk artifact cache directory
* `CCACHE_CACHE_SIZE` maximum size of the local on disk cache i.e. `10G`

#### s3 compatible

* `CCACHE_BUCKET` s3 bucket to be used
* `CCACHE_ENDPOINT` s3 endpoint
* `CCACHE_REGION` s3 region
* `CCACHE_S3_USE_SSL` s3 endpoint requires TLS, set this to `true`

The endpoint used then becomes `${CCACHE_BUCKET}.s3-{CCACHE_REGION}.amazonaws.com`.
If `CCACHE_REGION` is undefined, it will default to `us-east-1`.

#### cloudflare r2

* `CCACHE_BUCKET` is the name of your R2 bucket.
* `CCACHE_ENDPOINT` must follow the format of `https://<ACCOUNT_ID>.r2.cloudflarestorage.com`. Note that the `https://` must be included. Your account ID can be found [here](https://developers.cloudflare.com/fundamentals/get-started/basic-tasks/find-account-and-zone-ids/).
* `CCACHE_REGION` should be set to `auto`.

#### redis

* `CCACHE_REDIS` full redis url, including auth and access token/passwd

The full url appears then as `redis://user:passwd@1.2.3.4:6379/1`.

#### memcached

* `CCACHE_MEMCACHED` memcached url

#### gcs

* `CCACHE_GCS_BUCKET`
* `CCACHE_GCS_CREDENTIALS_URL`
* `CCACHE_GCS_KEY_PATH`
* `CCACHE_GCS_RW_MODE`

#### azure

* `CCACHE_AZURE_CONNECTION_STRING`

#### gha

* `CCACHE_GHA_CACHE_URL` / `ACTIONS_CACHE_URL` GitHub Actions cache API URL
* `CCACHE_GHA_RUNTIME_TOKEN` / `ACTIONS_RUNTIME_TOKEN` GitHub Actions access token
* `CCACHE_GHA_CACHE_TO` cache key to write
* `CCACHE_GHA_CACHE_FROM` comma separated list of cache keys to read from
