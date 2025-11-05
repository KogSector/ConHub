import featureToggles from '@/../feature-toggles.json'

export const isFeatureEnabled = (feature: string): boolean => {
  return featureToggles[feature as keyof typeof featureToggles] === true
}

export const isLoginEnabled = (): boolean => {
  const env = process.env.NEXT_PUBLIC_AUTH_ENABLED
  if (typeof env !== 'undefined') {
    return env === 'true' || env === '1'
  }
  // Fallback to feature toggles JSON (Auth flag)
  return isFeatureEnabled('Auth')
}

export const isHeavyModeEnabled = (): boolean => {
  return isFeatureEnabled('Heavy');
}
