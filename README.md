![mymy logo](logo.png)

[![AGPL License](https://img.shields.io/badge/license-AGPL-blue.svg)](http://www.gnu.org/licenses/agpl-3.0)

**mymy** is a user-friendly command-line application designed to help users gather information about their system quickly, intuitively, and easily. Instead of using multiple tools to access various system details, this single tool consolidates all the important information you need.

## Features

The main commands available in the mymy are:
- `ips`: Find out all the IP addresses allocated to your system, including local and external ones.
- `dns`: Discover your system's configured DNS server.
- `date`: Consult your system's configured date in a human-readable format.
- `time`: Consult your system's configured time and get the offset from the central NTP clock server.
- `datetime`: A combination of the `date` and `time` commands.
- `hostname`: Retrieve your system's hostname.
- `username`: Find out your current user's system username.
- `device-name`: Get your device's configured name.
- `os`: Identify the operating system your system is running.
- `architecture`: Determine your CPU's architecture.
- `interfaces`: List all the network interfaces configured on your system.

## Benefits

mymy eliminates the need to remember multiple commands and their specific syntax, as well as searching through different files for specific information. Additionally, this tool is multi-platform, making it even more convenient for users across different operating systems (MacOS, Linux, Windows).

## Installation

### Homebrew

```bash
brew tap oleiade/tap
brew install mymy
```

### Debian/Ubuntu

```bash
# Download and install the repository's GPG key
curl -fsSL https://oleiade.github.io/deb/oleiade-archive-keyring.gpg | gpg --dearmor > /usr/share/keyrings/oleiade-archive-keyring.gpg

# Add the repository to your system's sources
echo "deb [signed-by=/usr/share/keyrings/oleiade-archive-keyring.gpg] https://oleiade.github.io/deb stable main" > /etc/apt/sources.list.d/oleiade.list

# Update your sources
apt update

# Install mymy
apt install mymy
```

### Cargo

```fish
cargo install mymy
```

### Example Usage

Here are some example usages of the command:

```fish
$ my ips
public	84.172.75.134
local	192.168.2.246

$ my dns
1.1.1.1
8.8.8.8
192.168.1.1

$ my date
Saturday, 8 April, 2023, week 14

$ my time
20:51:42 UTC +02:00
±0.0795 seconds

$ my datetime
Saturday, 8 April, 2023, week 14
20:51:53 UTC +02:00
±0.0801 seconds

$ my hostname
oleiades-laptop.local

$ my username
oleiade

$ my device-name
Oleiade Laptop

$ my os
macOS 13.2.1

$ my architecture
arm64

$ my interfaces
lo0 127.0.0.1
lo0 ::1
en0 192.168.2.242
```

# Contributing

We appreciate your interest in contributing to our project! This is a small, open-source Rust project, and we welcome contributions from developers of all skill levels. To ensure a smooth and enjoyable experience for everyone involved, please take a moment to read through these guidelines before getting started.

## Getting Started

1. **Fork the repository**: Start by forking the project to your own GitHub account. This will create a personal copy of the repository that you can work on.
2. **Clone the repository**: Clone your forked repository to your local machine. You can do this by running the following command:
```bash
git clone https://github.com/your-username/mymy.git
```
3. **Create a new branch**: Create a new branch for your changes. Keep the branch name descriptive and concise. For example:
```bash
git checkout -b feature/add-new-command
```
4. **Make your changes**: Implement the new feature or fix the bug you've identified. Remember to follow the project's coding style and conventions.
5. **Commit your changes**: Once you've made your changes, commit them with a clear and descriptive commit message. This helps other contributors understand the purpose of your changes.
```bash
git commit -m "Add a new command for displaying system memory usage"
```
6. **Push your changes**: Push your changes to your forked repository on GitHub.
```bash
git push origin feature/add-new-command
```
7. **Submit a pull request**: Finally, create a pull request from your forked repository to the main project repository. Provide a clear and concise description of the changes you've made and the purpose of your pull request.

## Contribution Best Practices

- Always work on a new branch starting from the `develop` branch when making changes. Avoid making changes directly to the `main` branch.
- Keep your pull requests focused on a single feature or bugfix. If you have multiple unrelated changes, submit separate pull requests for each.
- Make sure your code is properly formatted and follows the project's coding style and conventions.
- Write clear and concise commit messages that describe the purpose of your changes.
- If you're fixing a bug, please provide a detailed description of the bug and steps to reproduce it.

## Reporting Bugs or Requesting Features

If you encounter any bugs or have a feature request, please open a new issue on the project's GitHub page. Be sure to provide a clear and concise description of the issue or feature request, and include any relevant information, such as error messages or steps to reproduce the issue.

## License

This project is licensed under the AGPL-3.0 license. For more information, see the [LICENSE](LICENSE) file.
