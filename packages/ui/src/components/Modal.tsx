import React from 'react'

interface ModalProps {
  isOpen: boolean
  onClose: () => void
  title?: string
  children: React.ReactNode
}

export function Modal({ isOpen, onClose, title, children }: ModalProps) {
  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />
      <div className="relative bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 border border-gray-700">
        {title && <h2 className="text-xl font-bold mb-4 text-white">{title}</h2>}
        {children}
      </div>
    </div>
  )
}
