const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const iconsDir = path.join(__dirname, 'src-tauri', 'icons');

// Icon sizes needed by Tauri
const sizes = [
  { size: 32, name: '32x32.png' },
  { size: 64, name: '64x64.png' },
  { size: 128, name: '128x128.png' },
  { size: 256, name: '128x128@2x.png' }, // 2x retina
];

// Load the source image
const sourceImage = path.join(iconsDir, 'kimi-logo.png');

async function generateIcons() {
  console.log('Generating icons from Kimi logo...');
  
  // Ensure source image exists
  if (!fs.existsSync(sourceImage)) {
    console.error('Source image not found:', sourceImage);
    process.exit(1);
  }

  // Generate PNG icons
  for (const { size, name } of sizes) {
    const outputPath = path.join(iconsDir, name);
    await sharp(sourceImage)
      .resize(size, size)
      .png()
      .toFile(outputPath);
    console.log(`Generated: ${name} (${size}x${size})`);
  }

  // Also save as icon.png (512x512 or largest)
  const iconPath = path.join(iconsDir, 'icon.png');
  await sharp(sourceImage)
    .resize(512, 512)
    .png()
    .toFile(iconPath);
  console.log('Generated: icon.png (512x512)');

  // Copy to app-icon.png
  const appIconPath = path.join(iconsDir, 'app-icon.png');
  await sharp(sourceImage)
    .resize(512, 512)
    .png()
    .toFile(appIconPath);
  console.log('Generated: app-icon.png');

  console.log('\nIcon generation complete!');
}

generateIcons().catch(console.error);
