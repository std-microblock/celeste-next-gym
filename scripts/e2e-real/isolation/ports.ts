import net from 'node:net'

import type { PortReservation } from '../types.js'

export const LOOPBACK = '127.0.0.1'

export async function reserveLoopbackPort(): Promise<PortReservation> {
  const server = net.createServer()
  await new Promise<void>((resolveListen, reject) => {
    server.once('error', reject)
    server.listen({ host: LOOPBACK, port: 0, exclusive: true }, resolveListen)
  })
  const address = server.address()
  if (address === null || typeof address === 'string') {
    server.close()
    throw new Error('failed to reserve a loopback TCP port')
  }
  let released = false
  return {
    port: address.port,
    async release(): Promise<void> {
      if (released) return
      released = true
      await new Promise<void>((resolveClose, reject) => {
        server.close((error) => error ? reject(error) : resolveClose())
      })
    },
  }
}
