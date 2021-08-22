# codec
A Swiss Army knife for en/de- coding/crypting strings

## Description
`codec` will read string from `stdin`, transform it through multiple en/de-coders
and print the result to `stdout`

## Usage
```
codec [options] [codecs]

options:
    -e (default): set the global coding mode to encode
    -d: set the global coding mode to decode
    -n: append new line ('\n') at the end of output (= append `newline` after)
    -I string: use `string` as input instead of stdin (= insert `const -C` before)
    -F file: use content of `file` as input instead of stdin (= insert `cat -c -F` before)
    -O file: use `file` as output instead of stdout (= append `tee -O` after)
    -h: print usage and exit

codecs:
    a list of **codec**s(en/de-coders), input will be passed and transformed from
    left to right (i.e. the output for a codec is the input for the next)

codec:
    codec-name [codec-options]

codec-options:
    lower case options are switch(boolean) options, so they take no argument.

    upper case options take one argument. the argument can be provided with plain
    string or by sub-codecs syntax: [plain-string codecs].

sub-codecs syntax:
    [ plain-string codecs ]

    The codecs inside [] will be run on `plain-string` as input, and the
    output is used as the argument.
```

### Examples
You can use `echo -n '' | ` to pass the input string directly.
```
codec -d base64 zlib
```
Decode base64 on input, and then decompress with zlib.

```
codec aes-ecb -K [12345678901234561234567890123456 hex -d] base64
```
Decode hex string `12345678901234561234567890123456`, and set it as aes-ecb key.
Encrypt the input, and then encode with base64. Note that unlike `openssl`, aes
codecs do not expect hex string as key. You always pass a raw byte string as key.

### Available Codecs and Options
If `-d` or `-e` is passed as a codec option, it will overwrite the global coding
mode.

```
Available codecs:
aes-cbc
    -K key
    -IV iv

aes-ecb
    -K key

append
    -A string: pass input to output, and then append `string`

base64
    -u: use url base64 instead

cat
    (if with no argument, behave like `id`)
    -c: (close input) do not read from input
    -F file: also read from `file`, optional

const
    -C replacement: ingore input, and replace the output with `replacement`

drop
    -B count: drop at most first `count` bytes from input

escape
    escape/unescape with shell-like quoting string escaping sequences

hex
    binary to hex encode or inverse
    -c: use capital hex string (only affects encoding)

id
    pass input to output as is
md5
    calculate hash digest

newline
    (= append -A ['\n' escape -d])
    append new line

redirect
    = tee -c -O `file`
    -O file: redirect output to `file`

repeat
    -T times: repeat input for `times` times (int, >=0, default 0)
rsa-crypt
    rsa encryption with public key and decryption with private key
    -PK pub_key: public key pem string, default pkcs1 format
    -SK pri_key: private key pem string, default pkcs1 format
    -8: use pkcs8 key format instead of pkcs1
    -PS scheme: padding scheme (oaep, pkcs15; defaults to oaep)
    -H algorithm: hash algorithm used for oaep padding scheme (sha1, sha256; defaults to sha256)

rsa-sign
    rsa sign with private key and verification with public key
    NOTE:
        1. input must first be hashed in algorithm specified in -H option
            e.g. sha256 rsa-sign -SK sk_string -H sha256
        2. for verification, output nothing if succeeded, error if not (pending to change along with new `if` meta codec)
    -PK pub_key: public key pem string, default pkcs1 format
    -SK pri_key: private key pem string, default pkcs1 format
    -8: use pkcs8 key format instead of pkcs1
    -H algorithm: hash algorithm used for sign (sha1, sha256)

sha256
    calculate hash digest

sink
    (= tee -c or redirect -O /dev/null on unix-like systems)
    differences with repeat: repeat without arguments (=repeat -T 0) will end the
    execution of the whole chain immediately, e.g.:
    const -C example tee -O /dev/stdout sink
        will output example
    const -C example tee -O /dev/stdout repeat
        will output nothing

sm3
    calculate hash digest

sm4-cbc
    -K key
    -IV iv

sm4-ecb
    -K key

system
   execute command, pipe its stdin as input, stdout as output
    -C command: command to run
    -A args: args for command

take
    -B count: take up to first `count` bytes from input

tee
    (if with no argument, behave like `id`)
    -c: (close output) do not write to output
    -O file: also write to `file`, optional

url
    url query escape/unescape
    -p: use path escape instead of query escape
usage
zlib
    -L level: compress level (int, [0, 9], default 6)
```

# TODO
1. refactor code
2. bug fixes in parser
3. usage
4. codec aliases/scripts (implement it as super-meta codec)
5. new codecs: i.e. if
6. plugins