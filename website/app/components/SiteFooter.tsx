import Link from "next/link";
import { ArrowUpRight } from "lucide-react";
import { githubUrl } from "../../lib/features";

export function SiteFooter() {
  return (
    <footer className="site-footer shell">
      <div>
        <Link className="brand" href="/">
          <img src="/suncode-mark-24-dark.svg" width="26" height="26" alt="" />
          <span>SunCode</span>
        </Link>
        <p>Your project. A clear view of the work.</p>
      </div>
      <nav aria-label="Footer navigation">
        <Link href="/features">All features</Link>
        <a href={githubUrl} target="_blank" rel="noreferrer">
          Source code <ArrowUpRight size={14} aria-hidden="true" />
        </a>
        <a href={`${githubUrl}/issues`} target="_blank" rel="noreferrer">
          Feedback <ArrowUpRight size={14} aria-hidden="true" />
        </a>
      </nav>
      <span className="footer-note">Built in the open.</span>
    </footer>
  );
}
