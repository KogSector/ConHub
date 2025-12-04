import featureToggles from '@/../feature-toggles.json'

export const isFeatureEnabled = (feature: string): boolean => {
  return featureToggles[feature as keyof typeof featureToggles] === true
}

export const isDockerEnabled = (): boolean => {
  return isFeatureEnabled('Docker');
}

export const isProdEnabled = (): boolean => {
  return isFeatureEnabled('Prod');
}
