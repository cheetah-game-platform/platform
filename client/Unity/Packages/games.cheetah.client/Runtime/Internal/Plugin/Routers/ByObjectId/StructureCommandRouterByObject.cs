using Games.Cheetah.Client.Internal.Plugin.Routers.FFI;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Internal.Plugin.Routers.ByObjectId
{
    public class StructureCommandRouterByObject : AbstractRouterByObject<CheetahBuffer>
    {
        private StructCommandRouter structCommandRouter;

        public override void Init(CheetahClient client)
        {
            base.Init(client);
            structCommandRouter = client.GetPlugin<StructCommandRouter>();
            structCommandRouter.ChangeListener += OnChange;
        }

        private void OnChange(ushort commandCreator, in CheetahObjectId objectId, ushort fieldId, ref CheetahBuffer value)
        {
            Notify(commandCreator, in objectId, fieldId, ref value);
        }
    }
}