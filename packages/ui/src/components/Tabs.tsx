import React from 'react'

interface TabsProps {
  defaultValue: string
  children: React.ReactNode
}

export function Tabs({ defaultValue, children }: TabsProps) {
  const [activeTab, setActiveTab] = React.useState(defaultValue)

  return (
    <div data-active-tab={activeTab}>
      {React.Children.map(children, (child) => {
        if (React.isValidElement(child)) {
          return React.cloneElement(child as any, { activeTab, setActiveTab })
        }
        return child
      })}
    </div>
  )
}

export function TabsList({ children, activeTab, setActiveTab }: any) {
  return (
    <div className="flex gap-2 border-b border-gray-700 pb-2">
      {React.Children.map(children, (child) => {
        if (React.isValidElement(child)) {
          return React.cloneElement(child as any, { activeTab, setActiveTab })
        }
        return child
      })}
    </div>
  )
}

export function TabsTrigger({ value, children, activeTab, setActiveTab }: any) {
  const isActive = activeTab === value

  return (
    <button
      onClick={() => setActiveTab(value)}
      className={`px-4 py-2 rounded transition ${
        isActive ? 'bg-blue-600 text-white' : 'text-gray-300 hover:bg-gray-700'
      }`}
    >
      {children}
    </button>
  )
}

export function TabsContent({ value, children, activeTab }: any) {
  if (activeTab !== value) return null

  return <div className="mt-4">{children}</div>
}
