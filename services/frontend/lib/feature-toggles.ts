import featureToggles from '@/../feature-toggles.json'

export const isFeatureEnabled = (feature: string): boolean => {
  return featureToggles[feature as keyof typeof featureToggles] === true
}

export const isLoginEnabled = (): boolean => {
  return true 
}

export const isHeavyModeEnabled = (): boolean => {
  return isFeatureEnabled('Heavy');
}
