import { useState } from "react";
import { QrReader } from "react-qr-reader";
import { router } from "./Router";

const QRCodeScanner: React.FC = () => {
  const [error, setError] = useState<string | null>(null);

  return (
    <div className="min-h-screen bg-gradient-to-br from-green-50 to-blue-100 flex items-center justify-center">
      <div className="w-full max-w-md bg-white rounded-lg shadow-lg p-4">
        <h1 className="text-2xl font-bold text-gray-900 mb-4 text-center">Scan QR Code</h1>
        <QrReader
          constraints={{ facingMode: "environment" }}
          onResult={(result, qrError) => {
            if (result) {
              const text = result.getText();
              try {
                const url = new URL(text);
                const match = /^\/scan\/([\w-]+)$/.exec(url.pathname);
                if (url.host === window.location.host && match) {
                  setError(null);
                  const token = match[1];
                  router.navigate({ to: "/scan/$token", params: { token } });
                } else {
                  setError("Invalid QuickRes QR code");
                }
              } catch {
                setError("Invalid QR code");
              }
            }
          }}
          videoStyle={{ width: "100%" }}
        />
        {error && <p className="text-red-600 mt-4 text-center">{error}</p>}
      </div>
    </div>
  );
};

export default QRCodeScanner;
