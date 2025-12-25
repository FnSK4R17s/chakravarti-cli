#!/usr/bin/env node

/**
 * Chakravarti CLI binary downloader
 * Downloads the correct pre-compiled binary for the current platform
 */

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const os = require('os');
const zlib = require('zlib');

const PACKAGE = require('./package.json');
const VERSION = PACKAGE.version;

// GitHub release configuration
const GITHUB_OWNER = 'chakravarti';
const GITHUB_REPO = 'cli';
const BINARY_NAME = 'ckrv';

// Platform mappings
const PLATFORM_MAP = {
    darwin: 'apple-darwin',
    linux: 'unknown-linux-gnu',
    win32: 'pc-windows-msvc',
};

const ARCH_MAP = {
    x64: 'x86_64',
    arm64: 'aarch64',
};

function getPlatformTarget() {
    const platform = os.platform();
    const arch = os.arch();

    const platformSuffix = PLATFORM_MAP[platform];
    const archPrefix = ARCH_MAP[arch];

    if (!platformSuffix || !archPrefix) {
        throw new Error(
            `Unsupported platform: ${platform}-${arch}. ` +
            `Supported: darwin-x64, darwin-arm64, linux-x64, linux-arm64, win32-x64`
        );
    }

    return `${archPrefix}-${platformSuffix}`;
}

function getBinaryUrl() {
    const target = getPlatformTarget();
    const ext = os.platform() === 'win32' ? '.exe' : '';
    const archiveExt = os.platform() === 'win32' ? '.zip' : '.tar.gz';

    return {
        url: `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/download/v${VERSION}/${BINARY_NAME}-${target}${archiveExt}`,
        binaryName: `${BINARY_NAME}${ext}`,
        isZip: os.platform() === 'win32',
    };
}

function download(url) {
    return new Promise((resolve, reject) => {
        const redirects = 5;

        function doRequest(url, remainingRedirects) {
            https.get(url, (response) => {
                if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
                    if (remainingRedirects <= 0) {
                        reject(new Error('Too many redirects'));
                        return;
                    }
                    doRequest(response.headers.location, remainingRedirects - 1);
                    return;
                }

                if (response.statusCode !== 200) {
                    reject(new Error(`Download failed: HTTP ${response.statusCode}`));
                    return;
                }

                const chunks = [];
                response.on('data', (chunk) => chunks.push(chunk));
                response.on('end', () => resolve(Buffer.concat(chunks)));
                response.on('error', reject);
            }).on('error', reject);
        }

        doRequest(url, redirects);
    });
}

async function extractTarGz(buffer, destDir, binaryName) {
    const tar = require('tar');
    const tmpFile = path.join(os.tmpdir(), `ckrv-${Date.now()}.tar.gz`);

    fs.writeFileSync(tmpFile, buffer);

    await tar.x({
        file: tmpFile,
        cwd: destDir,
        filter: (path) => path.endsWith(binaryName),
    });

    fs.unlinkSync(tmpFile);
}

async function extractZip(buffer, destDir, binaryName) {
    const AdmZip = require('adm-zip');
    const zip = new AdmZip(buffer);

    zip.getEntries().forEach((entry) => {
        if (entry.entryName.endsWith(binaryName)) {
            zip.extractEntryTo(entry, destDir, false, true);
        }
    });
}

async function install() {
    console.log(`Installing Chakravarti CLI v${VERSION}...`);

    const binDir = path.join(__dirname, 'bin');
    if (!fs.existsSync(binDir)) {
        fs.mkdirSync(binDir, { recursive: true });
    }

    try {
        const { url, binaryName, isZip } = getBinaryUrl();
        console.log(`Downloading from: ${url}`);

        const buffer = await download(url);
        console.log(`Downloaded ${(buffer.length / 1024 / 1024).toFixed(2)} MB`);

        const binaryPath = path.join(binDir, binaryName);

        if (isZip) {
            await extractZip(buffer, binDir, binaryName);
        } else {
            // For tar.gz, we'll use a simple approach
            const gunzip = zlib.gunzipSync(buffer);

            // Simple tar extraction (binary is usually the only file)
            // Skip tar header (512 bytes) and find the file data
            let offset = 0;
            while (offset < gunzip.length) {
                const header = gunzip.slice(offset, offset + 512);
                if (header[0] === 0) break;

                const name = header.slice(0, 100).toString('utf8').replace(/\0/g, '');
                const sizeOctal = header.slice(124, 136).toString('utf8').replace(/\0/g, '').trim();
                const size = parseInt(sizeOctal, 8) || 0;

                offset += 512;

                if (name.includes(BINARY_NAME) && !name.endsWith('/')) {
                    const fileData = gunzip.slice(offset, offset + size);
                    fs.writeFileSync(binaryPath, fileData);
                    fs.chmodSync(binaryPath, 0o755);
                    console.log(`Installed: ${binaryPath}`);
                    return;
                }

                offset += Math.ceil(size / 512) * 512;
            }

            throw new Error('Binary not found in archive');
        }

        // Make executable on Unix
        if (os.platform() !== 'win32') {
            fs.chmodSync(binaryPath, 0o755);
        }

        console.log(`✓ Chakravarti CLI installed successfully!`);
        console.log(`  Run 'ckrv --help' to get started.`);

    } catch (error) {
        console.error(`\n⚠️  Binary download failed: ${error.message}`);
        console.error(`\nAlternative installation methods:`);
        console.error(`  1. Install Rust and build from source:`);
        console.error(`     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`);
        console.error(`     cargo install chakravarti-cli`);
        console.error(`\n  2. Download manually from GitHub releases:`);
        console.error(`     https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases`);
        process.exit(1);
    }
}

install();
