import Link from 'next/link';
export default function NotFound() { return <div className="workspace empty-state"><h1>Page not found.</h1><p>This page is not part of the Robinhood Chain workspace.</p><Link className="button primary" href="/">Back to RustAG</Link></div>; }
