# khonsu

`khonsu` is a small dynamic DNS client that synchronises a dynamically assigned IP (such as  on a home router) with a DNS service. 

It similar to [ddclient](https://github.com/ddclient/ddclient) and [inadyn](https://github.com/troglobit/inadyn) but with the following key differences: 
* a single binary with no external dependencies
* does not execute as a system daemon, can run as an unprivileged user

## Usage

Typically executed from `cron` (checks and updates the IP every hour):

```
0 * * * * RUST_LOG=debug khonsu --config ~/.khonsu.toml
```

## Configuration

Configuration is via a TOML file with two primary sections.

An example file:

```toml
[detect.http]
uri = "https://ifconfig.co/ip"

[service.cloudflare]
api_token = "<apitoken>"
zone_id = "<zoneid>"
record_type = "A"
name = "<record name>"
```

### detect

Detection can use an external HTTP service such as `ifconfig.co` or a script.

#### `detect.http`

HTTP configuration supports two parameters:
* `uri` - (mandatory) the URI to fetch from your router
* `regex` - (optional) a regular expression that extracts the IP from the content

#### `detect.cmd`

Command / script configuration supports a single parameter:
* `cmd` - (mandatory) the script to execute

A simple option is to ssh into a remote host and echo the content of the `SSH_CLIENT` variable. 

For example, this script on the remote host:
 
```bash
#!/bin/bash

echo $SSH_CLIENT | cut -f1 -d' '
```

would be configured as such:
```toml
[detect.cmd]
cmd = "ssh host ~/bin/ip.sh"
```

### service

DNS services are configured under the `[service]` path.

#### `service.cloudflare`

Cloudflare requires the following settings:
* `api_token` - (mandatory) the API token for your Cloudflare account 
* `zone_id` - (mandatory) the zone ID from the Cloudflare dashboard
* `record_type` - (mandatory) the type of record synchronised by khonsu (`A` = ipv4, `AAAA` = ipv6)
* `name` - (mandatory) name of the DNS record to synchronise
