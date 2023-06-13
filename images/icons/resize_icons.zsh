#!/bin/zsh

# Source for pixel sizes: https://developer.apple.com/design/human-interface-guidelines/app-icons#macOS-app-icon-sizes
# Source for expected pixel sizes: https://docs.rs/tauri-icns/latest/tauri_icns/enum.IconType.html#variant.RGBA32_128x128_2x

# Input icon file
input_icon="images/icons/folsum_icon_660.png"

# Output directory for resized icons
output_dir="images/icons/resized_icons/"

# Ensure that the output directory exists.
mkdir ${output_dir}

# Array of @1x pixel icon sizes (in pixels) for macOS
sizes=(16 32 128 256 512)

# Loop through each @1x size and resize the icon
for size in "${sizes[@]}"; do
  output_file="$output_dir/folsum_icon_${size}px.png"
  convert "$input_icon" -resize "${size}x${size}" "$output_file"
done

# Array of @2x pixel icon sizes (in pixels) for macOS
sizes=(32 64 128 256 512 1024)

# Loop through each @2x size and resize the icon
for size in "${sizes[@]}"; do
  output_file="$output_dir/folsum_icon_${size}px@2x.png"
  convert "$input_icon" -resize "${size}x${size}" "$output_file"
done

