# shrine
Secrets manager written in rust.

![Rust](https://img.shields.io/github/languages/top/cpollet/shrine?color=orange)
[![CI](https://github.com/cpollet/shrine/actions/workflows/test.yml/badge.svg?branch=master)](https://github.com/cpollet/shrine/actions/workflows/test.yml)
[![License: Apache 2.0](https://img.shields.io/badge/licence-Apache%202.0-blue)](LICENSE)

# Command-line usage

### Initialize your shrine
```sh
shrine init
```

### Add secrets
```shell
shrine set personal/github mypassword
shrine set personal/email/me@myhost.net mySecurePassword
shrine set personal/email/me@gmail.com mySecurePassword
```

### Get a secret value
```shell
shrine get personal/github
```

### List secrets
```shell
shrine ls
shrine ls personal/email/.*
```

### Delete secrets
```shell
shrine rm personal/email/me@myhost.net
```

## Configure git integration
```shell
shrine config set git.enabled false
shrine config set git.commit.auto false
```
