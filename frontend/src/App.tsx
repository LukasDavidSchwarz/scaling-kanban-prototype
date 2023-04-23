import React from 'react';
import './App.css';
import { Route, Routes } from 'react-router-dom';
import Board from './Board';
import BoardList from './BoardList';
import Header from './Header';
import Footer from './Footer';

export default function App() {
    return (
        <div className="container-fluid h-100">
            <div className="d-flex flex-column h-100">
                <Header />
                <Routes>
                    <Route path="/" element={<BoardList />} />
                    <Route path="/boards/:boardId" element={<Board />} />
                </Routes>
                <Footer />
            </div>
        </div>
    );
}
