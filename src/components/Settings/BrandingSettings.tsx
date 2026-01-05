import { useSettingsStore } from '../../stores/settingsStore';

export function BrandingSettings() {
  const { vendorName, vendorUrl, vendorEmail, setVendorName, setVendorUrl, setVendorEmail } = useSettingsStore();

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-text-primary mb-1">Plugin Branding</h3>
        <p className="text-sm text-text-muted">
          Configure the vendor information that will be embedded in your VST/CLAP plugins.
        </p>
      </div>

      <div className="space-y-4">
        <div>
          <label htmlFor="vendorName" className="block text-sm font-medium text-text-secondary mb-1.5">
            Vendor Name
          </label>
          <input
            type="text"
            id="vendorName"
            value={vendorName}
            onChange={(e) => setVendorName(e.target.value)}
            placeholder="freqlab"
            className="w-full px-3 py-2 bg-bg-tertiary border border-border rounded-lg text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent"
          />
          <p className="mt-1 text-xs text-text-muted">
            This appears as the manufacturer name in DAWs.
          </p>
        </div>

        <div>
          <label htmlFor="vendorUrl" className="block text-sm font-medium text-text-secondary mb-1.5">
            Website URL
          </label>
          <input
            type="url"
            id="vendorUrl"
            value={vendorUrl}
            onChange={(e) => setVendorUrl(e.target.value)}
            placeholder="https://example.com"
            className="w-full px-3 py-2 bg-bg-tertiary border border-border rounded-lg text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent"
          />
          <p className="mt-1 text-xs text-text-muted">
            Optional. Your plugin's homepage or support page.
          </p>
        </div>

        <div>
          <label htmlFor="vendorEmail" className="block text-sm font-medium text-text-secondary mb-1.5">
            Contact Email
          </label>
          <input
            type="email"
            id="vendorEmail"
            value={vendorEmail}
            onChange={(e) => setVendorEmail(e.target.value)}
            placeholder="contact@example.com"
            className="w-full px-3 py-2 bg-bg-tertiary border border-border rounded-lg text-text-primary placeholder-text-muted focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent"
          />
          <p className="mt-1 text-xs text-text-muted">
            Optional. For plugin support inquiries.
          </p>
        </div>
      </div>

      <div className="pt-4 border-t border-border">
        <div className="bg-bg-tertiary rounded-lg p-4">
          <h4 className="text-sm font-medium text-text-primary mb-2">Preview</h4>
          <div className="text-xs text-text-muted space-y-1 font-mono">
            <p>VENDOR: &quot;{vendorName || 'freqlab'}&quot;</p>
            <p>URL: &quot;{vendorUrl || '(not set)'}&quot;</p>
            <p>EMAIL: &quot;{vendorEmail || '(not set)'}&quot;</p>
            <p>CLAP_ID: &quot;com.{(vendorName || 'freqlab').toLowerCase().replace(/[^a-z0-9]/g, '')}.&lt;plugin-name&gt;&quot;</p>
          </div>
        </div>
      </div>
    </div>
  );
}
