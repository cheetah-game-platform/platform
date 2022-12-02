using System.Threading.Tasks;
using Cheetah.Matches.Realtime.GRPC.Admin;
using Cheetah.Platform;
using UnityEditor;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer.Provider
{
    /// <summary>
    /// Провайдер команд с реального сервера
    /// </summary>
    public class RemoteDumpProvider : DumpProvider
    {
        private ClusterConnector connector;

        public RemoteDumpProvider(ClusterConnector connector)
        {
            this.connector = connector;
            EditorApplication.quitting += ApplicationQuitting;
        }

        public async Task<DumpResponse> Dump(ulong room)
        {
            return await connector.DoRequest(async channel =>
            {
                var client = new Dump.DumpClient(channel);
                return await client.DumpAsync(new DumpRequest
                {
                    Room = room
                });
            });
        }

        public async Task Destroy()
        {
            var tmpConnector = connector;
            connector = null;
            await tmpConnector.Destroy();
        }

        public async void ApplicationQuitting()
        {
            await Destroy();
        }
    }
}