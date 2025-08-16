import { useParams } from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";

const RetrieveReservation: React.FC = () => {
  const { reservationId } = useParams({ from: "/retrieve/$reservationId" });

  const { data, isLoading, error } = useQuery({
    queryKey: ["retrieve-reservation", reservationId],
    queryFn: () =>
      fetch(`http://localhost:8000/retrieve/${reservationId}`).then((res) =>
        res.json(),
      ),
  });

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">Loading reservation...</div>
    );
  }

  if (error || data?.error) {
    return (
      <div className="min-h-screen flex items-center justify-center">Error retrieving reservation</div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-purple-50 to-indigo-100 flex items-center justify-center">
      <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8 text-center">
        <h1 className="text-2xl font-bold text-gray-900 mb-4">Your Reservation</h1>
        <p className="mb-4">{data.event.name}</p>
        <div className="space-y-4">
          {data.reservation_tokens.map((token: { token: string; status: string }) => (
            <div key={token.token} className="flex flex-col items-center">
              <a href={`/scan/${token.token}`}>
                <img
                  src={`https://api.qrserver.com/v1/create-qr-code/?size=150x150&data=${encodeURIComponent(window.location.origin + '/scan/' + token.token)}`}
                  alt="Reservation QR"
                />
              </a>
              <p className="mt-2 text-sm text-gray-600">Status: {token.status}</p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default RetrieveReservation;
