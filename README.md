# codec
A Swiss Army knife for en/de- coding/crypting strings

## Description
`codec` will read string from `stdin`, transform it through multiple en/de-coders
and print the result to `stdout`

## Usage
```
codec options codecs

options:
    -e (default): set the global coding mode to encode
    -d: set the global coding mode to decode
    -n: append new line ('\n') at the end of output (= append `newline` after)
    -I string: use `string` as input instead of stdin (= insert `const -C` before)
    -F file: use content of `file` as input instead of stdin (= insert `cat -c -F` before)
    -O file: use `file` as output instead of stdout (= append `tee -O` after)

codecs:
    a list of codecs(en/de-coders), input will be passed and transformed from
    left to right

codec:
    codec-name codec-options

codec-options:
    lower case options are switch(boolean) options, so they take no argument.

    upper case options take one argument. the argument can be provided with plain
    string or by sub-codecs syntax: [plain-string codecs]. If you use sub-codecs
    syntax, the codecs inside [] will be run on `plain-string` as input, and the
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
url
    url query escape/unescape
    -p: use path escape instead of query escape

base64
    -u: use url base64 instead

aes-ecb
    -K key

aes-cbc
    -K key
    -IV iv

hex
    binary to hex encode or inverse
    -c: use capital hex string (only affects encoding)

sha256

md5

zlib
    -L level: compress level (int, [-2, 9], default -1)

id
    pass input to output as is

const
    -C replacement: ingore input, and replace the output with `replacement`

repeat
    -T times: repeat input for `times` times (int, >=0, default 0)

tee
    (if with no argument, behave like `id`)
    -c: (close output) do not write to output
    -O file: also write to `file`, optional

redirect
    = tee -c -O `file`
    -O file: redirect output to `file`

sink
    (= tee -c or redirect -O /dev/null on unix-like systems)
    differences with repeat: repeat without arguments (=repeat -T 0) will end the
    execution of the whole chain immediately, e.g.:
    const -C example tee -O /dev/stdout sink
        will output example
    const -C example tee -O /dev/stdout repeat
        will output nothing

append
    -A string: pass input to output, and then append `string`

newline
    (= append -A ['\n' escape -d])
    append new line

escape
    escape/unescape with shell-like quoting string escaping sequences

cat
    (if with no argument, behave like `id`)
    -c: (close input) do not read from input
    -F file: also read from `file`, optional

drop
    -B count: drop at most first `count` bytes from input

take
    -B count: take up to first `count` bytes from input
```

# TODO
1. refactor code
2. bug fixes in parser
3. usage
4. codec aliases/scripts (implement it as super-meta codec)
5. new codecs: i.e. if, system