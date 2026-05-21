import { useEffect } from 'react';
import { useAssetStore } from '../../store/assetStore';
import { AssetItem } from './AssetItem';

export function AssetBrowser() {
  const { projectRoot, assets, loading, error, refreshAssets } = useAssetStore();

  useEffect(() => {
    if (projectRoot) refreshAssets();
  }, [projectRoot]);

  if (!projectRoot) {
    return (
      <div className="asset-browser asset-browser--empty">
        <p>No project open.</p>
        <p className="muted">File → Open Project to load a folder.</p>
      </div>
    );
  }

  if (loading) return <div className="asset-browser"><span className="muted">Loading…</span></div>;
  if (error) return <div className="asset-browser"><span className="error">{error}</span></div>;

  return (
    <div className="asset-browser">
      <div className="asset-browser__toolbar">
        <span className="asset-browser__root" title={projectRoot}>
          {projectRoot.split('/').pop()}
        </span>
        <button className="icon-btn" onClick={refreshAssets} title="Refresh">↺</button>
      </div>
      <ul className="asset-list">
        {assets.map((asset) => (
          <AssetItem key={asset.id} asset={asset} />
        ))}
        {assets.length === 0 && (
          <li className="asset-list__empty">No assets found</li>
        )}
      </ul>
    </div>
  );
}
