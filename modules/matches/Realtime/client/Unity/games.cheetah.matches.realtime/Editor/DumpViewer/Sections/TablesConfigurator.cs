using Cheetah.Matches.Realtime.Editor.DumpViewer.TypesExtension;
using Cheetah.Matches.Realtime.Editor.UIElements.Table;
using Cheetah.Matches.Realtime.GRPC.Admin;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer.Sections
{
    public class TablesConfigurator
    {
        public static void ConfigureUsersTable(TableElement table)
        {
            table.AddColumn("Идентификатор", 200, 0, 1, o =>
            {
                var user = (DumpUser)o;
                return user.Id.ToString();
            });
            table.AddColumn("Группа", 200, 200, null, o =>
            {
                var user = (DumpUser)o;
                return user.Groups.ToString();
            });
            table.AddColumn("Присоеденен", 100, 100, null, o =>
            {
                var user = (DumpUser)o;
                return user.Attached.ToString();
            });
            table.SetSelectItemComparer(new DumpUserIdComparator());
            table.SetOrderComparer(new DumpUserIdComparator());
        }

        public static void ConfigureObjectsTable(TableElement table)
        {
            table.AddColumn("Идентификатор", 200, 200, null, (item) =>
            {
                var dumpObject = item as DumpObject;
                return GetDumpObjectId(dumpObject);
            });
            table.AddColumn("Группа", 120, null, null, (item) => (item as DumpObject).Groups.ToString());
            table.AddColumn("Шаблон", 200, null, null, (item) =>
            {
                var dumpObject = (item as DumpObject);
                return dumpObject.Template.ToString();
            });
            table.AddColumn("Создан", 100, null, null, (item) => (item as DumpObject).Created.ToString());
            table.SetSelectItemComparer(new DumpObjectIdComparator());
            table.SetOrderComparer(new DumpObjectIdComparator());
        }

        public static string GetDumpObjectId(DumpObject dumpObject)
        {
            return dumpObject.HasOwnerUserId ? "user(" + dumpObject.OwnerUserId + "," + dumpObject.Id + ")" : "Room(" + dumpObject.Id + ")";
        }
    }
}