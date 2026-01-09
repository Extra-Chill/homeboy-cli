class Homeboy < Formula
  desc "CLI tool for development and deployment automation"
  homepage "https://github.com/Extra-Chill/homeboy-cli"
  version "0.1.0"
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/Extra-Chill/homeboy-cli/releases/download/v0.1.0/homeboy-aarch64-apple-darwin.tar.xz"
      sha256 "c07d5b46612c2c36c523d3ea8df93d0623953b8175a3dd127c7cd975f326a0fa"
    end
    if Hardware::CPU.intel?
      url "https://github.com/Extra-Chill/homeboy-cli/releases/download/v0.1.0/homeboy-x86_64-apple-darwin.tar.xz"
      sha256 "c0e70eb8eec6f8375547491b0002be4de7e53ccba9e01d70d8205a486d3172e9"
    end
  end
  if OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/Extra-Chill/homeboy-cli/releases/download/v0.1.0/homeboy-aarch64-unknown-linux-gnu.tar.xz"
      sha256 "fd08f293e39fa76b836408a8dfb89e2264931ef7af5c4698593aab0e1c8e6660"
    end
    if Hardware::CPU.intel?
      url "https://github.com/Extra-Chill/homeboy-cli/releases/download/v0.1.0/homeboy-x86_64-unknown-linux-gnu.tar.xz"
      sha256 "6e06a06d3a7ed1dd6fafba6ceec8882611e5f4b5ca812d6f89692703461049c1"
    end
  end
  license "MIT"

  BINARY_ALIASES = {
    "aarch64-apple-darwin": {},
    "aarch64-unknown-linux-gnu": {},
    "x86_64-apple-darwin": {},
    "x86_64-pc-windows-gnu": {},
    "x86_64-unknown-linux-gnu": {}
  }

  def target_triple
    cpu = Hardware::CPU.arm? ? "aarch64" : "x86_64"
    os = OS.mac? ? "apple-darwin" : "unknown-linux-gnu"

    "#{cpu}-#{os}"
  end

  def install_binary_aliases!
    BINARY_ALIASES[target_triple.to_sym].each do |source, dests|
      dests.each do |dest|
        bin.install_symlink bin/source.to_s => dest
      end
    end
  end

  def install
    if OS.mac? && Hardware::CPU.arm?
      bin.install "homeboy"
    end
    if OS.mac? && Hardware::CPU.intel?
      bin.install "homeboy"
    end
    if OS.linux? && Hardware::CPU.arm?
      bin.install "homeboy"
    end
    if OS.linux? && Hardware::CPU.intel?
      bin.install "homeboy"
    end

    install_binary_aliases!

    # Homebrew will automatically install these, so we don't need to do that
    doc_files = Dir["README.*", "readme.*", "LICENSE", "LICENSE.*", "CHANGELOG.*"]
    leftover_contents = Dir["*"] - doc_files

    # Install any leftover files in pkgshare; these are probably config or
    # sample files.
    pkgshare.install(*leftover_contents) unless leftover_contents.empty?
  end
end
