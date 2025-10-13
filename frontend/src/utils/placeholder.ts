/**
 * Generate a local placeholder image as a data URI
 * @param text - Text to display on the placeholder
 * @param color - Background color (hex without #)
 * @param textColor - Text color (hex without #)
 * @param width - Width of the image
 * @param height - Height of the image
 * @returns Data URI for the SVG placeholder
 */
export function generatePlaceholder(
  text: string,
  color: string = '999999',
  textColor: string = 'ffffff',
  width: number = 300,
  height: number = 300
): string {
  const svg = `
    <svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg">
      <rect width="100%" height="100%" fill="#${color}"/>
      <text
        x="50%"
        y="50%"
        dominant-baseline="middle"
        text-anchor="middle"
        font-family="Arial, sans-serif"
        font-size="24"
        fill="#${textColor}"
      >${text}</text>
    </svg>
  `.trim()

  return `data:image/svg+xml;base64,${btoa(svg)}`
}

/**
 * Predefined placeholders for common fruit/vegetable categories
 */
export const categoryPlaceholders = {
  apple: generatePlaceholder('Apple', '67c23a', 'ffffff'),
  orange: generatePlaceholder('Orange', 'e6a23c', 'ffffff'),
  banana: generatePlaceholder('Banana', 'f9d71c', 'ffffff'),
  pear: generatePlaceholder('Pear', '67c23a', 'ffffff'),
  grape: generatePlaceholder('Grape', '722ed1', 'ffffff'),
  watermelon: generatePlaceholder('Watermelon', 'ff4d4f', 'ffffff'),
  other: generatePlaceholder('Other', '999999', 'ffffff')
}

/**
 * Get placeholder image for a category
 * @param category - Category name
 * @returns Data URI for the placeholder image
 */
export function getCategoryPlaceholder(category: string): string {
  const normalizedCategory = category.toLowerCase().replace(/\s+/g, '')

  if (normalizedCategory in categoryPlaceholders) {
    return categoryPlaceholders[normalizedCategory as keyof typeof categoryPlaceholders]
  }

  // Generate a placeholder with the category name
  const colors = ['67c23a', 'e6a23c', 'f9d71c', '722ed1', 'ff4d4f', '409eff']
  const colorIndex = category.length % colors.length
  return generatePlaceholder(category, colors[colorIndex], 'ffffff')
}
