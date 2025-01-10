#!/bin/zsh

# Source for pixel sizes: https://developer.apple.com/design/human-interface-guidelines/app-icons#macOS-app-icon-sizes
# Source for expected pixel sizes: https://docs.rs/tauri-icns/latest/tauri_icns/enum.IconType.html#variant.RGBA32_128x128_2x

# Input icon file
input_icon="images/icons/folsum_icon_660.png"

# Output directory for resized icons
output_dir="images/icons/resized_icons/"

# Ensure that the output directory exists.
mkdir ${output_dir}

# Array of @1x (non-retina) pixel icon sizes for macOS.
display_sizes=(16 32 128 256 512)

# Loop through each size and resize the icon.
for display_size in "${display_sizes[@]}"; do
  # Name the resized icon after its size.
  output_file="$output_dir/folsum_icon_${display_size}px.png"
  # Resize teh 
  convert "$input_icon" -resize "${display_size}x${display_size}" "$output_file"

  # Double the display size to get the equivalent retina size.
  retina_size=$((display_size * 2))
  # Use the display size to name the retina sized icon and add @2x the stem.
  output_file="$output_dir/folsum_icon_${display_size}px@2x.png"
  convert "$input_icon" -resize "${retina_size}x${retina_size}" "$output_file"
done

