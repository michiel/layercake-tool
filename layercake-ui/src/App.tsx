import { Layout, Menu, theme } from 'antd';
import { Routes, Route, Link, useLocation } from 'react-router-dom';
import ProjectsList from './components/ProjectsList';
import ProjectDetail from './components/ProjectDetail';
import GraphDetail from './components/GraphDetail';
import PlanDetail from './components/PlanDetail';
import NotFound from './components/NotFound';

const { Header, Content, Footer } = Layout;

function App() {
  const { token } = theme.useToken();
  const location = useLocation();

  const menuItems = [
    { key: '/', label: <Link to="/">Projects</Link> },
  ];

  return (
    <Layout>
      <Header style={{ position: 'sticky', top: 0, zIndex: 1, width: '100%' }}>
        <div className="logo" />
        <Menu
          theme="dark"
          mode="horizontal"
          selectedKeys={[location.pathname]}
          items={menuItems}
        />
      </Header>
      <Content className="site-layout-content" style={{ padding: '0 50px' }}>
        <div style={{ padding: 24, minHeight: 'calc(100vh - 64px - 69px)', background: token.colorBgContainer, borderRadius: token.borderRadiusLG }}>
          <Routes>
            <Route path="/" element={<ProjectsList />} />
            <Route path="/projects/:projectId" element={<ProjectDetail />} />
            <Route path="/projects/:projectId/graph" element={<GraphDetail />} />
            <Route path="/projects/:projectId/plan" element={<PlanDetail />} />
            <Route path="*" element={<NotFound />} />
          </Routes>
        </div>
      </Content>
      <Footer style={{ textAlign: 'center' }}>
        Layercake Tool ©{new Date().getFullYear()} - GraphQL Visualization Tool
      </Footer>
    </Layout>
  );
}

export default App;
