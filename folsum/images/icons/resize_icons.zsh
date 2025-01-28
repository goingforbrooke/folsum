#!/bin/zsh

# If you want to run this script manually, then run it from `folsum/folsum/`.

# Input icon file.
input_icon="images/icons/folsum_icon_660.png"

# Output directory for resized icons for MacOS applications.
output_dir="images/icons/resized_icons/"

# Ensure that the output directory exists.
mkdir ${output_dir}

## MacOS Icons

# Source for pixel sizes: https://developer.apple.com/design/human-interface-guidelines/app-icons#macOS-app-icon-sizes
# Source for expected pixel sizes: https://docs.rs/tauri-icns/latest/tauri_icns/enum.IconType.html#variant.RGBA32_128x128_2x

# Array of @1x (non-retina) pixel icon sizes for macOS.
display_sizes=(16 32 128 256 512)

# Loop through each size and resize the icon.
for display_size in "${display_sizes[@]}"; do
  # Name the resized icon after its size.
  output_file="$output_dir/folsum_icon_${display_size}px.png"
  # Resize the base icon to the desired size.
  magick "$input_icon" -resize "${display_size}x${display_size}" "$output_file"

  # Double the display size to get the equivalent retina size.
  retina_size=$((display_size * 2))
  # Use the display size to name the retina sized icon and add @2x the stem.
  output_file="$output_dir/folsum_icon_${display_size}px@2x.png"
  magick "$input_icon" -resize "${retina_size}x${retina_size}" "$output_file"
done

## Web Icons

# Source for web icons: https://evilmartians.com/chronicles/how-to-favicon-in-2021-six-files-that-fit-most-needs
#- ✅favicon.ico in 32x32 for legacy browsers
#- ❌SVG (inherently with no size) for modern browsers
#  - use inkscape to magick PNG export to SVG b/c doing Paths in GIMP sucks
#    - export to PNG
#    - Path -> Trace Bitmap…
#      - Pixel Art tab on the right
#        - Export as…
#          - PNG
#        - alpha background will look white in firefox
#          - use XnView to check for alpha background
#- ✅180x180 PNG for Apple devices
#- ✅192x192 PNG and 512x512 PNG for Android devices

# Output directory for web icons.
web_icons_dir="images/icons/web_icons/"

# Ensure that the output directory exists.
mkdir ${web_icons_dir}

web_icon_sizes=(180 192 512)

# Loop through each size and resize the web icon.
for web_icon_size in "${web_icon_sizes[@]}"; do
  echo "Web icon sizes: ${web_icon_sizes[@]}"

  # Name the resized icon after its size.
  output_file="$web_icons_dir/folsum_icon_${web_icon_size}px.png"
  # Resize the base icon to the desired size.
  magick "$input_icon" -resize "${web_icon_size}x${web_icon_size}" "$output_file";
  echo "Done converting to ${web_icon_size}x${web_icon_size}"
done

# Create a 32x32 ICO.
output_file="$web_icons_dir/favicon.ico"
magick "$input_icon" -resize "32x32" "$output_file"
