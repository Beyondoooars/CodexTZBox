import { onMounted } from 'vue'

export function useAppearance() {
  function applyLightAppearance() {
    document.documentElement.dataset.macvueAppearance = 'light'
    document.documentElement.style.colorScheme = 'light'
  }

  onMounted(() => {
    applyLightAppearance()
  })
}
