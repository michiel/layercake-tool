import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { ApolloProvider } from '@apollo/client/react'
import { apolloClient } from './graphql/client'
import App from './App'
import { Toaster } from '@/components/ui/sonner'
import { ThemeProvider } from '@/components/theme/theme-provider'
import { TagsFilterProvider } from '@/hooks/useTagsFilter'
import { TooltipProvider } from '@/components/ui/tooltip'

// Tailwind CSS
import './index.css'

const root = ReactDOM.createRoot(document.getElementById('root')!)

root.render(
  <React.StrictMode>
    <ApolloProvider client={apolloClient}>
      <Toaster />
      <TooltipProvider>
        <BrowserRouter>
          <ThemeProvider
            attribute="class"
            defaultTheme="light"
            enableSystem={false}
            storageKey="layercake-theme"
          >
            <TagsFilterProvider>
              <App />
            </TagsFilterProvider>
          </ThemeProvider>
        </BrowserRouter>
      </TooltipProvider>
    </ApolloProvider>
  </React.StrictMode>
)
