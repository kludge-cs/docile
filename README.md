# 📄 Docile

A command-line tool to tame structured file formats for programmatic input.

## 🛠️ Installation

### Cargo

```sh
$ cargo install
```

### Nix

#### Declarative

```nix
environment.systemPackages = [
  inputs.docile.packages.<arch>.docile
];
```

#### Imperative

```sh
$ nix profile install github:kludge-cs/docile
```

## 📝 Usage

```sh
# Using `proselint` as an example
$ docile thesis.pdf | proselint
```

## 🧩 Development

```sh
$ nix develop # If Nix
```