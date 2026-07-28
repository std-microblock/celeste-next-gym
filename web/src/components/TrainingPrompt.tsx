import { useEffect, useRef, useState, type CSSProperties } from 'react'
import type { GymMap, SimState, Vec2 } from '../model'

const MAX_SPEED = 900
const ACCELERATION = 5_400
const TYPE_INTERVAL_MS = 28
const PROMPT_ANCHOR_ABOVE_PLAYER = 20

function approach(current: number, target: number, amount: number): number {
  return current < target ? Math.min(target, current + amount) : Math.max(target, current - amount)
}

export function promptTargetPercent(map: GymMap, state: SimState, viewport: { width: number; height: number }): Vec2 {
  if (viewport.width <= 0 || viewport.height <= 0) return { x: 50, y: 50 }
  const scale = Math.min(viewport.width / map.bounds.width, viewport.height / map.bounds.height)
  const contentWidth = map.bounds.width * scale
  const contentHeight = map.bounds.height * scale
  const offsetX = (viewport.width - contentWidth) / 2
  const offsetY = (viewport.height - contentHeight) / 2
  return {
    x: (offsetX + (state.pos.x - map.bounds.x) * scale) / viewport.width * 100,
    y: (offsetY + (state.pos.y - map.bounds.y - PROMPT_ANCHOR_ABOVE_PLAYER) * scale) / viewport.height * 100,
  }
}

export function TrainingPrompt({ map, state, viewport, text, hidden = false }: { map: GymMap; state: SimState; viewport: { width: number; height: number }; text: string; hidden?: boolean }) {
  const target = promptTargetPercent(map, state, viewport)
  const position = useRef<Vec2>(target)
  const velocity = useRef<Vec2>({ x: 0, y: 0 })
  const targetRef = useRef(target)
  const [renderedPosition, setRenderedPosition] = useState(target)
  const [visibleLength, setVisibleLength] = useState(0)

  targetRef.current = target

  useEffect(() => {
    setVisibleLength(0)
    if (hidden) return
    const timer = window.setInterval(() => {
      setVisibleLength((length) => {
        if (length >= text.length) {
          window.clearInterval(timer)
          return length
        }
        return length + 1
      })
    }, TYPE_INTERVAL_MS)
    return () => window.clearInterval(timer)
  }, [hidden, text])

  useEffect(() => {
    let animation = 0
    let last = performance.now()
    const tick = (now: number) => {
      const delta = Math.min(.05, (now - last) / 1000)
      last = now
      const desiredX = targetRef.current.x - position.current.x
      const desiredY = targetRef.current.y - position.current.y
      const distance = Math.hypot(desiredX, desiredY)
      const desiredSpeed = Math.min(MAX_SPEED, distance * 14)
      const desiredVelocity = distance <= .01 ? { x: 0, y: 0 } : { x: desiredX / distance * desiredSpeed, y: desiredY / distance * desiredSpeed }
      velocity.current = {
        x: approach(velocity.current.x, desiredVelocity.x, ACCELERATION * delta),
        y: approach(velocity.current.y, desiredVelocity.y, ACCELERATION * delta),
      }
      position.current = {
        x: position.current.x + velocity.current.x * delta,
        y: position.current.y + velocity.current.y * delta,
      }
      setRenderedPosition(position.current)
      animation = requestAnimationFrame(tick)
    }
    animation = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(animation)
  }, [])

  if (hidden) return null
  return <div
    className="training-prompt"
    style={{ '--prompt-x': `${renderedPosition.x}%`, '--prompt-y': `${renderedPosition.y}%` } as CSSProperties}
  >
    {text.slice(0, visibleLength)}<i aria-hidden="true" />
  </div>
}
