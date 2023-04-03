# mymy

[![AGPL License](https://img.shields.io/badge/license-AGPL-blue.svg)](http://www.gnu.org/licenses/agpl-3.0)

Access the most common information about your system using a single command.

Mymy is a command line tool that provides the most helpful information about your system in a single command. You won't need to remember which command to use or which file to consult to get the information you need. Mymy will do it for you. 

We started this project because, as engineers working on three different operating systems daily, remembering which command to use or which file to open to find out about standard system information was a pain. We wanted to have a single command that would give us the information we needed without having to remember anything.

## Features

Using the `my` command, you can get the following information about your system:
- your current IP address using the `my ip` command
- your system's configured DNS servers using the `my dns` command
- your system's time, and its offset to a reference clock server using the `my time` command.
- your system's configured date using the `my date` command.
- a combination of your system's time and date using the `my datetime` command.

## Installation

```bash
    cargo install mymy
```
    
## Usage/Examples

```bash
> my ip
83.173.75.136

> my dns
8.8.8.8
1.1.1.1
192.168.1.1

> my time
09:35:59 +02:00
±0.0276 seconds

> my date
Monday, 03 April, 2023, week 14

> my datetime
09:36:28 +02:00, Monday, 03 April, 2023, week 14
±0.0277 seconds
```

## How to use

### IP address

To get your current IP address, use the `my ip` command.

```bash
> my ip
84.173.77.136
```

### DNS servers

To get your system's configured DNS servers, use the `my dns` command.

```bash
> my dns
8.8.8.8
1.1.1.1
192.168.1.1
```

### Time

To get your system's time, and its offset to a reference clock server, use the `my time` command.

```bash
> my time
09:35:59 +02:00
±0.0276 seconds
```

### Date

To get your system's configured date, use the `my date` command.

```bash
> my date
Monday, 03 April, 2023, week 14
```

### Date and time

To get a combination of your system's time and date, use the `my datetime` command.

```bash
> my datetime
09:36:28 +02:00, Monday, 03 April, 2023, week 14
±0.0277 seconds
```

## Contributing

Contributions are always welcome!

See `contributing.md` for ways to get started.
