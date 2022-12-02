using System.Collections;
using Cheetah.Matches.Realtime.GRPC.Admin;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer.TypesExtension
{
    internal class DumpObjectIdComparator : IComparer
    {
        public int Compare(object x, object y)
        {
            var o1 = (DumpObject)x;
            var o2 = (DumpObject)y;

            if (o1.HasOwnerUserId && !o2.HasOwnerUserId)
            {
                return -1;
            }

            if (o2.HasOwnerUserId && !o1.HasOwnerUserId)
            {
                return 1;
            }

            var owner = o1.OwnerUserId.CompareTo(o2.OwnerUserId);
            return owner == 0 ? o1.Id.CompareTo(o2.Id) : owner;
        }
    }
}