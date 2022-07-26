using Games.Cheetah.Client.Internal.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.ServerAPI.FFI
{
    public class EventServerAPI : IEventServerAPI
    {
        
        public byte SetListener(ushort clientId, IEventServerAPI.Listener listener)
        {
            return EventFFI.SetListener(clientId, listener);
        }

        public byte Send(ushort clientId, in CheetahObjectId objectId, FieldId.Event fieldId, ref CheetahBuffer data)
        {
            return EventFFI.Send(clientId, in objectId, fieldId.Id, ref data);
        }

        public byte Send(ushort clientId, ushort targetUser, in CheetahObjectId objectId, FieldId.Event fieldId, ref CheetahBuffer data)
        {
            return EventFFI.Send(clientId, targetUser, in objectId, fieldId.Id, ref data);
        }
    }
}