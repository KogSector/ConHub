import featureToggles from '@/../feature-toggles.json'

export const isFeatureEnabled = (feature: string): boolean => {
  return featureToggles[feature as keyof typeof featureToggles] === true
}

export const isLoginEnabled = (): boolean => {
  // Respect feature toggles JSON only; env overrides are ignored.
  return isFeatureEnabled('Auth')
}

export const isDockerEnabled = (): boolean => {
  return isFeatureEnabled('Docker');
}

export const isProdEnabled = (): boolean => {
  return isFeatureEnabled('Prod');
}
