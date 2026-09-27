import "./globals.css";

export const metadata = {
  title: "Proxy Tool",
  description: "Local API request inspector",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
