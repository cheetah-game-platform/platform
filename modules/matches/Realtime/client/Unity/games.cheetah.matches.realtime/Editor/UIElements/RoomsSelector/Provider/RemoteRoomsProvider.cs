using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime.GRPC.Admin;
using Cheetah.Platform;
using UnityEditor;

namespace Cheetah.Matches.Realtime.Editor.UIElements.RoomsSelector.Provider
{
    public class RemoteRoomsProvider : RoomsProvider
    {
        private ClusterConnector connector;

        public RemoteRoomsProvider(ClusterConnector connector)
        {
            this.connector = connector;
            EditorApplication.quitting += ApplicationQuitting;
        }

        public async Task<IList<ulong>> GetRooms()
        {
            var result = await connector.DoRequest(async (channel) =>
            {
                var client = new GRPC.Admin.Realtime.RealtimeClient(channel);
                return await client.GetRoomsAsync(new GetRoomsRequest());
            });
            return result.Rooms.ToList();
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

namespace Cheetah.Matches.Realtime.GRPC
{
    public interface Realtime
    {
    }
}