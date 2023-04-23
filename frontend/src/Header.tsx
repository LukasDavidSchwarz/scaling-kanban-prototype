import './Header.css';
import { Link } from 'react-router-dom';

export default function Header() {
    return (
        <div className="header row">
            <div className="btn">
                <Link to="/">
                    <h5 className="header-link font-weight-bold text-center m-0">
                        {import.meta.env.VITE_APP_TITLE}
                    </h5>
                </Link>
            </div>
        </div>
    );
}
