import { describe, expect, it } from 'vitest'
import { playerHairMetadata } from './playerHair'

describe('player hair metadata', () => {
  it('uses the duck offset for the unnumbered duck sprite', () => {
    expect(playerHairMetadata('characters/player/duck')).toEqual({
      offset: { x: 0, y: 3 },
      frame: 0,
    })
  })

  it('keeps numbered animation frame metadata', () => {
    expect(playerHairMetadata('characters/player/climb08')).toEqual({
      offset: { x: 2, y: -2 },
      frame: 2,
    })
  })
})
