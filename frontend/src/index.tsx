import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

const rootElement = document.getElementById('root') as HTMLElement;
if (!rootElement) throw new Error('Root container missing in index.html');

const root = ReactDOM.createRoot(rootElement);
// TODO: Use strict mode once react-beautiful-dnd supports it for react18
//  (see https://github.com/atlassian/react-beautiful-dnd/issues/2407)
//  or it is replaced by another drag and drop solution
root.render(<App />);
