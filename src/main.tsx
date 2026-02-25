import React from 'react';
import ReactDOM from 'react-dom/client';
import { ConfigProvider, App as AntdApp, theme } from 'antd';
import App from './App';
import './styles/global.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ConfigProvider
      theme={{
        algorithm: theme.darkAlgorithm,
        token: {
          colorPrimary: '#E74C3C',
          colorBgContainer: '#161B22',
          colorBgElevated: '#1C2333',
          colorBorder: '#30363D',
          colorText: '#E6EDF3',
          colorTextSecondary: '#8B949E',
          fontFamily: '"DIN Next", "DIN Alternate", system-ui, sans-serif',
          fontSize: 13,
          borderRadius: 6,
        },
      }}
    >
      <AntdApp>
        <App />
      </AntdApp>
    </ConfigProvider>
  </React.StrictMode>
);
