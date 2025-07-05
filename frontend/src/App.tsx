import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AppLayout } from '@/components/layout/AppLayout';
import { Dashboard } from '@/pages/Dashboard';
import { Projects } from '@/pages/Projects';
import { Plans } from '@/pages/Plans';
import { Graph } from '@/pages/Graph';

// Create a client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      refetchOnWindowFocus: false,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Router>
        <Routes>
          <Route path="/" element={<AppLayout />}>
            <Route index element={<Dashboard />} />
            <Route path="projects" element={<Projects />} />
            <Route path="projects/:projectId" element={<Projects />} />
            <Route path="projects/:projectId/plans" element={<Plans />} />
            <Route path="projects/:projectId/graph" element={<Graph />} />
            <Route path="plans" element={<div className="p-6">Plans - Coming Soon</div>} />
            <Route path="graphs" element={<div className="p-6">Graphs - Coming Soon</div>} />
            <Route path="analytics" element={<div className="p-6">Analytics - Coming Soon</div>} />
          </Route>
        </Routes>
      </Router>
    </QueryClientProvider>
  );
}

export default App;