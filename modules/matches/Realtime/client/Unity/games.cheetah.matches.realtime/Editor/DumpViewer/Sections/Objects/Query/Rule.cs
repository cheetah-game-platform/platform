using System.Collections.Generic;
using System.Linq;
using Cheetah.Matches.Realtime.GRPC.Admin;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer.Sections.Objects.Query
{
    /// <summary>
    /// Правило фильтрации игрового объекта
    /// </summary>
    public interface Rule
    {
        bool Filter(DumpObject dumpObject);

        /// <summary>
        /// Фильтрация поля, среди отфильтрованных объектов
        /// </summary>
        /// <param name="fieldItem"></param>
        /// <returns></returns>
        bool Filter(ObjectsViewer.FieldItem fieldItem);
    }

    /// <summary>
    /// Отрицание правила
    /// </summary>
    public class NotRule : Rule
    {
        private readonly Rule rule;

        public NotRule(Rule rule)
        {
            this.rule = rule;
        }

        public bool Filter(DumpObject dumpObject)
        {
            return !rule.Filter(dumpObject);
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return !rule.Filter(fieldItem);
        }

        protected bool Equals(NotRule other)
        {
            return Equals(rule, other.rule);
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            return obj.GetType() == typeof(NotRule) && Equals((NotRule)obj);
        }

        public override int GetHashCode()
        {
            return (rule != null ? rule.GetHashCode() : 0);
        }

        public override string ToString()
        {
            return GetType().Name + "(" + rule + ")";
        }
    }

    public class AndRule : Rule
    {
        private readonly IList<Rule> rules;

        public AndRule(IList<Rule> rules)
        {
            this.rules = rules;
        }

        public bool Filter(DumpObject dumpObject)
        {
            return rules.All(c => c.Filter(dumpObject));
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return rules.All(c => c.Filter(fieldItem));
        }

        public void AddRule(Rule rule)
        {
            if (rule is AndRule andRule)
            {
                foreach (var subRule in andRule.rules)
                {
                    rules.Add(subRule);
                }
            }
            else
            {
                rules.Add(rule);
            }
        }

        protected bool Equals(AndRule other)
        {
            return rules.SequenceEqual(other.rules);
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            return obj.GetType() == GetType() && Equals((AndRule)obj);
        }

        public override int GetHashCode()
        {
            return (rules != null ? rules.GetHashCode() : 0);
        }

        public override string ToString()
        {
            return GetType().Name + ": [" + string.Join(",", this.rules) + "]";
        }
    }

    public class OrRule : Rule
    {
        private readonly IList<Rule> rules;

        public OrRule(IList<Rule> rules)
        {
            this.rules = rules;
        }

        public void AddRule(Rule rule)
        {
            if (rule is OrRule orRule)
            {
                foreach (Rule stubRule in orRule.rules)
                {
                    rules.Add(stubRule);
                }
            }
            else
            {
                rules.Add(rule);
            }
        }

        public bool Filter(DumpObject dumpObject)
        {
            return rules.Any(c => c.Filter(dumpObject));
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return rules.Any(c => c.Filter(fieldItem));
        }

        protected bool Equals(OrRule other)
        {
            return rules.SequenceEqual(other.rules);
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            return obj.GetType() == GetType() && Equals((OrRule)obj);
        }

        public override int GetHashCode()
        {
            return (rules != null ? rules.GetHashCode() : 0);
        }

        public override string ToString()
        {
            return GetType().Name + ": [" + string.Join(",", this.rules) + "]";
        }
    }


    /// <summary>
    /// Поиск по шаблону
    /// </summary>
    public class TemplateRule : Rule
    {
        private readonly uint template;

        public TemplateRule(uint template)
        {
            this.template = template;
        }

        public bool Filter(DumpObject dumpObject)
        {
            return dumpObject.Template == template;
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return true;
        }

        protected bool Equals(TemplateRule other)
        {
            return template == other.template;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            return obj.GetType() == typeof(TemplateRule) && Equals((TemplateRule)obj);
        }

        public override int GetHashCode()
        {
            return (int)template;
        }

        public override string ToString()
        {
            return GetType().Name + "(" + template + ")";
        }
    }

    /// <summary>
    /// Поиск по группе доступа
    /// </summary>
    public class AccessGroupsRule : Rule
    {
        private readonly ulong groups;

        public AccessGroupsRule(ulong groups)
        {
            this.groups = groups;
        }

        public bool Filter(DumpObject dumpObject)
        {
            return dumpObject.Groups == groups;
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return true;
        }

        protected bool Equals(AccessGroupsRule other)
        {
            return groups == other.groups;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            return obj.GetType() == this.GetType() && Equals((AccessGroupsRule)obj);
        }

        public override int GetHashCode()
        {
            return groups.GetHashCode();
        }

        public override string ToString()
        {
            return GetType().Name + "(" + groups + ")";
        }
    }

    /// <summary>
    /// Поиск по признаку создания объекта
    /// </summary>
    public class CreatedFlagRule : Rule
    {
        private readonly bool created;

        public CreatedFlagRule(bool created)
        {
            this.created = created;
        }

        public bool Filter(DumpObject dumpObject)
        {
            return dumpObject.Created == created;
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return true;
        }

        protected bool Equals(CreatedFlagRule other)
        {
            return created == other.created;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            return obj.GetType() == GetType() && Equals((CreatedFlagRule)obj);
        }

        public override int GetHashCode()
        {
            return created.GetHashCode();
        }

        public override string ToString()
        {
            return GetType().Name + "(" + created + ")";
        }
    }

    /// <summary>
    /// Поиск по создателю - пользователю
    /// </summary>
    public class UserOwnerRule : Rule
    {
        private readonly uint user;

        public UserOwnerRule(uint user)
        {
            this.user = user;
        }

        public bool Filter(DumpObject dumpObject)
        {
            return dumpObject.HasOwnerUserId && dumpObject.OwnerUserId == user;
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return true;
        }

        protected bool Equals(UserOwnerRule other)
        {
            return user == other.user;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            return obj.GetType() == this.GetType() && Equals((UserOwnerRule)obj);
        }

        public override int GetHashCode()
        {
            return (int)user;
        }

        public override string ToString()
        {
            return $"User: {user}";
        }
    }

    /// <summary>
    /// Поиск объектов созданных комнатой
    /// </summary>
    public class RoomOwnerRule : Rule
    {
        public bool Filter(DumpObject dumpObject)
        {
            return !dumpObject.HasOwnerUserId;
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return true;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            return obj.GetType() == this.GetType();
        }


        public override int GetHashCode()
        {
            return 0;
        }


        public override string ToString()
        {
            return GetType().Name;
        }
    }

    /// <summary>
    /// Поиск по идентификатору объекта
    /// </summary>
    public class IdRule : Rule
    {
        private readonly uint id;

        public IdRule(uint id)
        {
            this.id = id;
        }

        public bool Filter(DumpObject dumpObject)
        {
            return dumpObject.Id == id;
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return true;
        }

        protected bool Equals(IdRule other)
        {
            return id == other.id;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            return obj.GetType() == GetType() && Equals((IdRule)obj);
        }

        public override int GetHashCode()
        {
            return (int)id;
        }

        public override string ToString()
        {
            return GetType().Name + "(" + id + ")";
        }
    }

    /// <summary>
    /// Поиск по идентификатору поля
    /// </summary>
    public class FieldIdRule : Rule
    {
        private readonly uint fieldId;

        public FieldIdRule(uint fieldId)
        {
            this.fieldId = fieldId;
        }

        public bool Filter(DumpObject dumpObject)
        {
            return dumpObject.Fields.Any(f => f.Id == fieldId);
        }

        public bool Filter(ObjectsViewer.FieldItem fieldItem)
        {
            return fieldItem.Id == fieldId;
        }

        protected bool Equals(FieldIdRule other)
        {
            return fieldId == other.fieldId;
        }

        public override bool Equals(object obj)
        {
            if (ReferenceEquals(null, obj)) return false;
            if (ReferenceEquals(this, obj)) return true;
            if (obj.GetType() != this.GetType()) return false;
            return Equals((FieldIdRule)obj);
        }

        public override int GetHashCode()
        {
            return (int)fieldId;
        }

        public override string ToString()
        {
            return GetType().Name + "(" + fieldId + ")";
        }
    }
}