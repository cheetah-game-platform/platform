using System.Runtime.InteropServices;

namespace Cheetah.Matches.Realtime.EmbeddedServer.FFI
{
    internal static class Server
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void OnServerError([MarshalAs(UnmanagedType.LPWStr)] string message);

        [StructLayout(LayoutKind.Sequential)]
        internal struct Description
        {
            [MarshalAs(UnmanagedType.U8)] internal ulong id;

            internal unsafe fixed byte gameIp[4];
            [MarshalAs(UnmanagedType.U2)] internal ushort gamePort;

            internal unsafe fixed byte internal_grpc_ip[4];
            [MarshalAs(UnmanagedType.U2)] internal ushort internal_grpc_port;

            internal unsafe fixed byte internal_webgrpc_ip[4];
            [MarshalAs(UnmanagedType.U2)] internal ushort internal_webgrpc_port;

            internal unsafe fixed byte admin_webgrpc_ip[4];
            [MarshalAs(UnmanagedType.U2)] internal ushort admin_webgrpc_port;
        }

        [StructLayout(LayoutKind.Sequential)]
        internal struct BindAddress
        {
            internal unsafe fixed byte bindAddress[4];
        }


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "run_new_server")]
        internal static extern bool RunNewServer(ref Description description, OnServerError onServerError,
            ref BindAddress bindAddress);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "destroy_server")]
        internal static extern bool DestroyServer(ulong serverId);
    }
}