import { useParams } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";

const ScanReservation: React.FC = () => {
  const { token } = useParams({ from: "/scan/$token" });

  const { data, isLoading, error } = useQuery({
    queryKey: ["scan-token", token],
    queryFn: () =>
      fetch(`http://localhost:8000/scan/${token}`).then((res) => res.json()),
  });

  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 to-orange-100 flex items-center justify-center">
      <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8 text-center">
        {isLoading ? (
          <h1 className="text-2xl font-bold text-gray-900 mb-4">Scanning...</h1>
        ) : error || data?.error ? (
          <h1 className="text-2xl font-bold text-red-600 mb-4">Invalid or used token</h1>
        ) : (
          <>
            <h1 className="text-2xl font-bold text-gray-900 mb-4">Checked In!</h1>
            <p className="text-gray-600">Token {data.token} marked as {data.status}</p>
          </>
        )}
      </div>
    </div>
  );
};

export default ScanReservation;
