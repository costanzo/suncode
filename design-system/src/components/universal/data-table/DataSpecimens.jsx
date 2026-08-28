export function DataSpecimens() {
  return <div className="specimen-grid specimen-grid-2">
    <div className="code"><span className="comment">// provider route</span><br /><span className="keyword">const</span> model = <span className="string">&quot;deepseek-v4-pro&quot;</span>;<br /><span className="keyword">const</span> path = <span className="string">&quot;agent/context.rs&quot;</span>;<br /><span className="keyword">await</span> runTests({`{ cwd: projectRoot }`});</div>
    <div className="table-wrap"><table className="data-table"><thead><tr><th>File</th><th>Change</th><th>Status</th></tr></thead><tbody><tr><td><strong className="mono">agent.rs</strong></td><td>+42 / -8</td><td><span className="badge success">Ready</span></td></tr><tr><td><strong className="mono">App.axaml</strong></td><td>+16 / -4</td><td><span className="badge accent">Review</span></td></tr><tr><td><strong className="mono">DESIGN.md</strong></td><td>+30 / -2</td><td><span className="badge warning">Pending</span></td></tr></tbody></table></div>
  </div>;
}
